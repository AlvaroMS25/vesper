use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;

mod after;
mod argument;
mod attr;
mod autocomplete;
mod before;
mod check;
mod command;
mod error_handler;
mod details;
mod hook;
mod modal;
mod optional;
mod parse;
mod util;

/// Converts an `async` function into a normal function returning a
/// `Pin<Box<dyn Future<Output = _> + '_>>`
#[doc(hidden)]
#[proc_macro_attribute]
pub fn hook(_: TokenStream, input: TokenStream) -> TokenStream {
    extract(hook::hook(input.into()))
}

/// Converts an `async-compatible` function into a builder and modifies function's body
/// to parse all required arguments, for further information about the behaviour of this macro, see
/// the implementation.
///
/// By an `async-compatible` function it's meant a function with a minimum of one argument,
/// which must be an `&SlashContext<T>`, which is always given and also used to parse all arguments.
///
/// # Usage:
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
/// ## Arguments:
///
/// You **must** provide another `description` attribute for every argument describing what they
/// are, this description will be seen on discord when filling up the argument. This needs to be
/// done with all the arguments except the context, which must be the first one, the accepted
/// syntax is the same as the previous `description` one.
///
/// ### Renaming:
/// Adding a `rename` attribute is optional, but can be used to modify the name of the argument seen
/// in discord, it is allowed to have only one `rename` attribute per argument and the attribute can
/// be used the same ways a the `description` one.
///
/// ### Autocompletion:
/// Adding an `autocomplete` attribute is also optional, but it allows the developer to complete
/// the user's input for an argument. This attribute is used the same way as the description one,
/// but it *must* point to a function marked with the `#[autocomplete]` attribute macro.
///
/// ## Specifying required permissions
///
/// It is possible to specify the permissions needed to execute the command by using the
/// `#[required_permissions]` attribute. It accepts as input a list of comma separated
/// [twilight permissions](https://docs.rs/twilight-model/latest/twilight_model/guild/struct.Permissions.html).
/// For example, to specify that a user needs to have administrator permissions to execute a command,
/// the attribute would be used like this `#[required_permissions(ADMINISTRATOR)]`.
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

#[proc_macro_attribute]
pub fn check(_: TokenStream, input: TokenStream) -> TokenStream {
    extract(check::check(input.into()))
}

#[proc_macro_attribute]
pub fn error_handler(_: TokenStream, input: TokenStream) -> TokenStream {
    extract(error_handler::error_handler(input.into()))
}

/// Prepares the function to be used to autocomplete command arguments.
#[proc_macro_attribute]
pub fn autocomplete(_: TokenStream, input: TokenStream) -> TokenStream {
    extract(autocomplete::autocomplete(input.into()))
}

#[proc_macro_derive(Parse, attributes(rename))]
pub fn parse(input: TokenStream) -> TokenStream {
    extract(parse::parse(input.into()))
}

/// Implements the `Modal` trait for the derived struct, allowing it to create modals and collect
/// the inputs provided by the user.
///
/// # Examples
///
/// ```rust
/// use zephyrus::prelude::*;
///
/// #[derive(Modal)]
/// struct MyModal {
///     something: String,
///     optional_item: Option<String>
/// }
/// ```
///
/// # Attributes
///
/// The derive macro accepts several attributes:
///
/// - `#[title = "<TITLE>"`: This attribute allows specifying the title of the modal, by default the
/// title will be the name of the structure.
///
/// ## Example
///
/// ```rust
/// #[derive(Modal)]
/// struct MyModal { // <- This modal will have "MyModal" as title.
///     //...
/// }
///
/// #[derive(Modal)]
/// #[title = "Some incredible modal"]
/// struct OtherModal { // <- This one will have "Some incredible modal" as the title.
///     // ...
/// }
/// ```
///
/// - `#[label = "<LABEL>"`: This attribute allows setting the label of the field, by default it will
/// be the name of the struct field.
///
/// ## Example
///
/// ```rust
/// use zephyrus::prelude::*;
///
/// #[derive(Modal)]
/// struct MyModal {
///     #[label = "My field"]
///     something: String, // <- This field will be shown as "My field"
///     optional_item: Option<String> // <- This one will use the struct name "optional_item"
/// }
/// ```
///
/// - `#[max_length = x]` || `#[min_length = y]`: These attributes allow to set a maximum/minimum
/// amount of characters a field can have.
///
/// ## Example
///
/// ```rust
/// use zephyrus::prelude::*;
///
/// #[derive(Modal)]
/// struct MyModal {
///     #[max_length = 150]
///     #[min_length = 15]
///     something: String, // <- This field will have both maximum and minimum size constraints.
///     #[max_length = 25]
///     short_field: String, // <- This field will only have a maximum size constraint.
///     optional_item: Option<String> // <- This one won't have any
/// }
/// ```
///
/// - `#[paragraph]`: This attribute will mark the field as a paragraph. By default, all fields are
/// marked as single line fields, so the user will only be able to input up to one line unless we
/// mark it as a paragraph.
///
/// ## Example
///
/// ```rust
/// use zephyrus::prelude::*;
///
/// #[derive(Modal)]
/// struct MyModal {
///     #[paragraph]
///     something: String, // <- This field will be shown as a multi-line field.
///     optional_item: Option<String> // <- This one will be shown as a single line one.
/// }
/// ```
#[proc_macro_derive(
    Modal,
    attributes(title, label, max_length, min_length, paragraph, placeholder)
)]
pub fn modal(input: TokenStream) -> TokenStream {
    extract(modal::modal(input.into()))
}

/// Extracts the given result, throwing a compile error if an error is given.
fn extract(res: syn::Result<TokenStream2>) -> TokenStream {
    match res {
        Ok(s) => s,
        Err(why) => why.to_compile_error(),
    }
    .into()
}
