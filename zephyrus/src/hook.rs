use crate::context::AutocompleteContext;
use crate::{
    context::SlashContext, twilight_exports::InteractionResponseData,
    BoxFuture,
};

/// A pointer to a function used by [before hook](BeforeHook).
pub(crate) type BeforeFn<D> = for<'a> fn(&'a SlashContext<'a, D>, &'a str) -> BoxFuture<'a, bool>;
/// A hook executed before command execution.
pub struct BeforeHook<D>(pub BeforeFn<D>);

/// A pointer to a function used by [after hook](AfterHook).
pub(crate) type AfterFn<D, T, E> =
    for<'a> fn(&'a SlashContext<'a, D>, &'a str, Option<Result<T, E>>) -> BoxFuture<'a, ()>;
/// A hook executed after command execution.
pub struct AfterHook<D, T, E>(pub AfterFn<D, T, E>);

/// A pointer to a function used by [autocomplete hook](AutocompleteHook)
pub(crate) type AutocompleteFn<D> =
    for<'a> fn(AutocompleteContext<'a, D>) -> BoxFuture<'a, Option<InteractionResponseData>>;
/// A hook used to suggest inputs to the command caller.
pub struct AutocompleteHook<D>(pub AutocompleteFn<D>);

pub(crate) type CheckFn<D, E> = for<'a> fn(&'a SlashContext<'a, D>) -> BoxFuture<'a, Result<bool, E>>;

pub struct CheckHook<D, E>(pub CheckFn<D, E>);

pub(crate) type ErrorHandlerFn<D, E> = for<'a> fn(&'a SlashContext<'a, D>, E) -> BoxFuture<'a, ()>;

pub struct ErrorHandlerHook<D, E>(pub ErrorHandlerFn<D, E>);
