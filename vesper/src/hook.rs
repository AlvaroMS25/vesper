use crate::context::AutocompleteContext;
use crate::{
    context::SlashContext, twilight_exports::InteractionResponseData,
    BoxFuture,
};

/// A pointer to a function used by [before hook](BeforeHook).
pub(crate) type BeforeFn<D> = for<'a> fn(&'a SlashContext<'a, D>, &'a str) -> BoxFuture<'a, bool>;
/// A hook executed before a command execution.
///
/// The function must have as parameters a [slash context] reference and a `&str`
/// which contains the name of the command to execute.
///
/// [slash context]: SlashContext
pub struct BeforeHook<D>(pub BeforeFn<D>);

/// A pointer to a function used by [after hook](AfterHook).
pub(crate) type AfterFn<D, T, E> =
    for<'a> fn(&'a SlashContext<'a, D>, &'a str, Option<Result<T, E>>) -> BoxFuture<'a, ()>;

/// A hook executed after a command execution.
///
/// The function must have as parameters a [slash context] reference, a `&str` which contains
/// the name of the command, and an `Option<Result<T, E>>`.
///
/// The result contained in the option must be the same as your command's output.
///
/// Note that it will be missing only if the command had an error and an error handler was set
/// to handle the error.
///
/// [slash context]: SlashContext
pub struct AfterHook<D, T, E>(pub AfterFn<D, T, E>);

/// A pointer to a function used by [autocomplete hook](AutocompleteHook).
pub(crate) type AutocompleteFn<D> =
    for<'a> fn(AutocompleteContext<'a, D>) -> BoxFuture<'a, Option<InteractionResponseData>>;

/// A hook used to suggest inputs to the command caller.
///
/// The function must have as parameter a single [autocomplete context](AutocompleteContext).
pub struct AutocompleteHook<D>(pub AutocompleteFn<D>);

/// A pointer to a function used by the [check hook](CheckHook).
pub(crate) type CheckFn<D, E> = for<'a> fn(&'a SlashContext<'a, D>) -> BoxFuture<'a, Result<bool, E>>;

/// A hook that can be used to determine if a command should execute or not depending
/// on the given function.
pub struct CheckHook<D, E>(pub CheckFn<D, E>);

/// A pointer to a function used by the [error handler hook](ErrorHandlerHook).
pub(crate) type ErrorHandlerFn<D, E> = for<'a> fn(&'a SlashContext<'a, D>, E) -> BoxFuture<'a, ()>;

/// A hook that can be used to handle errors of an specific command and its checks.
///
/// The function must have as parameters a [slash context] reference and the actual error type
/// the function and check is supposed to return.
///
/// [slash context]: SlashContext
pub struct ErrorHandlerHook<D, E>(pub ErrorHandlerFn<D, E>);
