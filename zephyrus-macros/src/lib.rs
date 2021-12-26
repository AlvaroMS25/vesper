use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;

mod after;
mod argument;
mod attr;
mod before;
mod command;
mod futurize;
mod options;
mod parse;
mod util;

/// Converts an `async` function into a normal function returning a
/// `Pin<Box<dyn Future<Output = _> + '_>>`
#[proc_macro_attribute]
pub fn futurize(_: TokenStream, input: TokenStream) -> TokenStream {
    extract(futurize::futurize(input.into()))
}

/// Converts an `async-compatible` function into a builder and modifies function's body
/// to parse all required commands, for further information about the behaviour of this macro, see
/// the implementation.
///
/// By an `async-compatible` function it's meant a function with a minimum of one argument,
/// which must be an `&SlashContext<T>`, which is always given and also used to parse all arguments.
///
/// Usage:
///
/// This macro can be used two ways:
///
///     - Without arguments, as #[command], which takes the caller function name as the name of the command.
///     - Providing the name, as #[command("command name")] which takes the provided name as the command name.
///
/// When marking a function with this attribute macro, you **must** provide a description of the
/// command that will be seen on discord when using the command, this is made by adding a
/// `description` attribute, which can be added two ways:
///
///     - List way: #[description("Some description")]
///
///     - Named value way: #[description = "Some description"]
///
/// Arguments:
///
/// You **must** provide another `description` attribute for every argument describing what they
/// are, this description will be seen on discord when filling up the argument. This needs to be
/// done with all the arguments except the context, which must be the first one, the accepted
/// syntax is the same as the previous `description` one.
///
/// Adding a `rename` attribute is optional, but can be used to modify the name of the argument seen
/// in discord, it is allowed to have only one `name` attribute per argument and the attribute can
/// be used the same ways a the `description` one.
#[proc_macro_attribute]
pub fn command(attrs: TokenStream, input: TokenStream) -> TokenStream {
    extract(command::command(attrs.into(), input.into()))
}

/// Prepares the function to allow it to be set as an after hook, see
/// the implementation for more information about this macro's behaviour.
#[proc_macro_attribute]
pub fn after(_: TokenStream, input: TokenStream) -> TokenStream {
    extract(after::after(input.into()))
}

/// Prepares the function to allow it to be set as a before hook, see
/// the implementation for more information about this macro's behaviour.
#[proc_macro_attribute]
pub fn before(_: TokenStream, input: TokenStream) -> TokenStream {
    extract(before::before(input.into()))
}

#[proc_macro_derive(Parse, attributes(rename))]
pub fn parse(input: TokenStream) -> TokenStream {
    extract(parse::parse(input.into()))
}

/// Extracts the given result, throwing a compile error if an error is given.
fn extract(res: syn::Result<TokenStream2>) -> TokenStream {
    match res {
        Ok(s) => s,
        Err(why) => why.to_compile_error(),
    }
    .into()
}
