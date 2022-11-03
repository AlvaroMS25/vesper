use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{
    parse2, spanned::Spanned, Error, FnArg, GenericParam, ItemFn, Lifetime, LifetimeDef, Result,
    ReturnType, Type,
};

/// The implementation of the hook macro, this macro takes the given function and changes
/// it's output and body to fit into a `Pin<Box<dyn Future>>`
pub fn hook(input: TokenStream2) -> Result<TokenStream2> {
    let fun = parse2::<ItemFn>(input)?;

    let ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = fun;

    if sig.asyncness.is_none() {
        /*
        In order to return a `Future` object, the function must be async, so if this function is
        not even async, this makes no sense to even try to execute this macro's function
        */
        return Err(Error::new(sig.span(), "Function must be marked async"));
    }

    // As we will change the return to return a Pin<Box<dyn Future>>
    // we don't need the function to be async anymore.
    sig.asyncness = None;

    // The output of the function as a token stream, so we can quote it after
    let output = match &sig.output {
        ReturnType::Default => quote::quote!(()),
        ReturnType::Type(_, t) => quote::quote!(#t),
    };

    // Wrap the original result in a Pin<Box<dyn Future>>
    sig.output = parse2(quote::quote!(
        -> ::std::pin::Pin<Box<dyn ::std::future::Future<Output = #output> + 'future + Send>>
    ))?;

    /*
    As we know all functions marked with this macro have an `SlashContext` reference, we have to
    add a lifetime which will be assigned to all references used in the function and to the returned
    future
    */
    sig.generics.params.insert(
        0,
        GenericParam::Lifetime(LifetimeDef {
            attrs: Default::default(),
            lifetime: Lifetime::new("'future", Span::call_site()),
            colon_token: None,
            bounds: Default::default(),
        }),
    );

    for i in sig.inputs.iter_mut() {
        if let FnArg::Typed(kind) = i {
            // If the argument is a reference, assign the previous defined lifetime to it
            if let Type::Reference(ty) = &mut *kind.ty {
                ty.lifetime = Some(Lifetime::new("'future", Span::call_site()));
            }
        }
    }

    Ok(quote::quote! {
        #(#attrs)*
        #vis #sig {
            Box::pin(async move #block)
        }
    })
}
