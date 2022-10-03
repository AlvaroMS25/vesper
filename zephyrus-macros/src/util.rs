use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{
    parse2, Error, FnArg, GenericArgument, Pat, PatType, Path, PathArguments, Result, ReturnType,
    Signature, Type,
};

/// Gets the path of the futurize macro
pub fn get_futurize_macro() -> Path {
    parse2(quote::quote!(::zephyrus::macros::futurize)).unwrap()
}

/// Gets the path of the command struct used internally by zephyrus
pub fn get_command_path() -> Path {
    parse2(quote::quote!(::zephyrus::command::Command)).unwrap()
}

/// Gets the path of the given type
pub fn get_path(t: &Type) -> Result<&Path> {
    match t {
        // If the type is actually a path, just return it
        Type::Path(p) => Ok(&p.path),
        // If the type is a reference, call this function recursively until we get the path
        Type::Reference(r) => get_path(&r.elem),
        _ => Err(Error::new(
            t.span(),
            "parameter must be a path to a context type",
        )),
    }
}

/// Gets the path of the given type
pub fn get_path_mut(t: &mut Type) -> Result<&mut Path> {
    match t {
        // If the type is actually a path, just return it
        Type::Path(p) => Ok(&mut p.path),
        // If the type is a reference, call this function recursively until we get the path
        Type::Reference(r) => get_path_mut(&mut r.elem),
        _ => Err(Error::new(
            t.span(),
            "parameter must be a path to a context type",
        )),
    }
}

/// Get the ascription pattern of the given function argument
pub fn get_pat(arg: &FnArg) -> Result<&PatType> {
    match arg {
        FnArg::Typed(t) => Ok(t),
        _ => Err(Error::new(
            arg.span(),
            "`self` parameter is not allowed here",
        )),
    }
}

/// Get the ascription pattern of the given function argument
pub fn get_pat_mut(arg: &mut FnArg) -> Result<&mut PatType> {
    match arg {
        FnArg::Typed(t) => Ok(t),
        _ => Err(Error::new(
            arg.span(),
            "`self` parameter is not allowed here",
        )),
    }
}

/// Gets the identifier of the given pattern
pub fn get_ident(p: &Pat) -> Result<Ident> {
    match p {
        Pat::Ident(pi) => Ok(pi.ident.clone()),
        _ => Err(Error::new(p.span(), "parameter must have an identifier")),
    }
}

/// Gets the generic arguments of the given path, this is useful to extract the generic parameter
/// used in `SlashContext<T>`
pub fn get_generic_arguments(path: &Path) -> Result<impl Iterator<Item = &GenericArgument> + '_> {
    match &path.segments.last().unwrap().arguments {
        PathArguments::None => Ok(Vec::new().into_iter()),
        PathArguments::AngleBracketed(arguments) => {
            Ok(arguments.args.iter().collect::<Vec<_>>().into_iter())
        }
        _ => Err(Error::new(
            path.span(),
            "context type cannot have generic parameters in parenthesis",
        )),
    }
}

/// Gets the identifier and the type of the first argument of a function, which must be an
/// `SlashContext`
pub fn get_context_type_and_ident(sig: &Signature) -> Result<(Ident, Type)> {
    let ctx = match sig.inputs.iter().next() {
        None => {
            return Err(Error::new(
                sig.inputs.span(),
                "Expected SlashContext as first paramenter",
            ))
        }
        Some(c) => get_pat(c)?,
    };

    let ctx_ident = get_ident(&ctx.pat)?;
    let path = get_path(&ctx.ty)?;
    let mut args = get_generic_arguments(path)?;

    let ty = loop {
        match args.next() {
            Some(GenericArgument::Lifetime(_)) => (),
            Some(a) => {
                break match a {
                    GenericArgument::Type(t) => {
                        if let Type::Infer(_) = t {
                            return Err(Error::new(
                                sig.inputs.span(),
                                "SlashContext must have a known type",
                            ));
                        } else {
                            t.clone()
                        }
                    }
                    _ => {
                        return Err(Error::new(
                            sig.inputs.span(),
                            "SlashContext type must be a type",
                        ))
                    }
                }
            }
            None => {
                return Err(Error::new(
                    sig.inputs.span(),
                    "SlashContext type must be set",
                ))
            }
        }
    };

    Ok((ctx_ident, ty))
}

/// Checks whether the given return type is the same as the provided one
pub fn check_return_type(ret: &ReturnType, out: TokenStream) -> Result<()> {
    let ty = match &ret {
        ReturnType::Default => syn::parse2::<Type>(quote::quote!(()))?,
        ReturnType::Type(_, ty) => syn::parse2::<Type>(quote::quote!(#ty))?,
    };

    let out = parse2(quote::quote!(#out))?;

    if ty != out {
        return Err(Error::new(
            ret.span(),
            format!(
                "Expected {} as return type, got {}",
                out.to_token_stream(),
                ty.to_token_stream()
            ),
        ));
    }

    Ok(())
}
