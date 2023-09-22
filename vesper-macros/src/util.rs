use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse2, Error, FnArg, GenericArgument, Pat, PatType, Path, PathArguments, Result, ReturnType, Signature, Type, Lifetime};
use crate::util;

/// Gets the path of the futurize macro
pub fn get_hook_macro() -> Path {
    parse2(quote::quote!(::zephyrus::macros::hook)).unwrap()
}

/// Gets the path of the command struct used internally by vesper
pub fn get_command_path() -> Path {
    parse2(quote::quote!(::zephyrus::command::Command)).unwrap()
}

pub fn get_returnable_trait() -> Path {
    parse2(quote::quote!(::zephyrus::extract::Returnable)).unwrap()
}

/// Gets the path of the given type
pub fn get_path(t: &Type, allow_references: bool) -> Result<&Path> {
    match t {
        // If the type is actually a path, just return it
        Type::Path(p) => Ok(&p.path),
        // If the type is a reference, call this function recursively until we get the path
        Type::Reference(r) => {
            if allow_references {
                get_path(&r.elem, allow_references)
            } else {
                Err(Error::new(r.span(), "Reference not allowed"))
            }
        },
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

/// Gets the generic arguments of the given path
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

pub fn get_return_type(sig: &Signature) -> Result<Box<Type>> {
    match &sig.output {
        ReturnType::Default => return Err(Error::new(sig.output.span(), "Return type must be a Result<T, E>")),
        ReturnType::Type(_, kind) => Ok(kind.clone())
    }
}

pub fn get_context_type(sig: &Signature, allow_references: bool) -> Result<Type> {
    let arg = match sig.inputs.iter().next() {
        None => {
            return Err(Error::new(
                sig.inputs.span(),
                "Expected Context as first parameter",
            ))
        }
        Some(c) => c,
    };

    let ty = util::get_bracketed_generic(arg, allow_references, |ty| {
        if let Type::Infer(_) = ty {
            Err(Error::new(ty.span(), "Context must have a known type"))
        } else {
            Ok(ty.clone())
        }
    })?;

    match ty {
        None => Err(Error::new(arg.span(), "Context type must be set")),
        Some(ty) => Ok(ty),
    }
}

pub fn set_context_lifetime(sig: &mut Signature) -> Result<()> {
    let lifetime = Lifetime::new("'future", Span::call_site());
    let ctx = get_pat_mut(sig.inputs.iter_mut().next().unwrap())?;
    let path = get_path_mut(&mut ctx.ty)?;
    let mut insert_lifetime = true;

    {
        let generics = util::get_generic_arguments(path)?;
        for generic in generics {
            if let GenericArgument::Lifetime(inner) = generic {
                if *inner == lifetime {
                    insert_lifetime = false;
                }
            }
        }
    }

    if insert_lifetime {
        if let PathArguments::AngleBracketed(inner) =
        &mut path.segments.last_mut().unwrap().arguments
        {
            inner.args.insert(0, GenericArgument::Lifetime(lifetime));
        }
    }

    Ok(())
}

pub fn get_bracketed_generic<F>(arg: &FnArg, allow_references: bool, fun: F) -> Result<Option<Type>>
where
    F: Fn(&Type) -> Result<Type>
{
    let mut generics = get_generic_arguments(get_path(&get_pat(arg)?.ty, allow_references)?)?;

    while let Some(next) = generics.next() {
        match next {
            GenericArgument::Lifetime(_) => (),
            GenericArgument::Type(ty) => return Ok(Some(fun(ty)?)),
            other => {
                return Err(Error::new(other.span(), "Generic must be a type"))
            }
        }
    }

    Ok(None)
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
