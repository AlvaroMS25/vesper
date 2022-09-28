use crate::context::AutocompleteContext;
use crate::{
    command::CommandResult, context::SlashContext, twilight_exports::InteractionResponseData,
    BoxFuture,
};

/// A pointer to a function used by [before hook](BeforeHook).
pub(crate) type BeforeFn<D> = for<'a> fn(&'a SlashContext<'a, D>, &'a str) -> BoxFuture<'a, bool>;
/// A hook executed before command execution.
pub struct BeforeHook<D>(pub BeforeFn<D>);

/// A pointer to a function used by [after hook](AfterHook).
pub(crate) type AfterFn<D> =
    for<'a> fn(&'a SlashContext<'a, D>, &'a str, CommandResult) -> BoxFuture<'a, ()>;
/// A hook executed after command execution.
pub struct AfterHook<D>(pub AfterFn<D>);

/// A pointer to a function used by [autocomplete hook](AutocompleteHook)
pub(crate) type AutocompleteFn<D> =
    for<'a> fn(AutocompleteContext<'a, D>) -> BoxFuture<'a, Option<InteractionResponseData>>;
/// A hook used to suggest inputs to the command caller.
pub struct AutocompleteHook<D>(pub AutocompleteFn<D>);
