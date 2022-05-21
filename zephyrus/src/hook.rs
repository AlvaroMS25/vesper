use crate::context::AutocompleteContext;
use crate::{
    command::CommandResult, context::SlashContext, twilight_exports::InteractionResponseData,
    BoxFuture,
};

/// A pointer to a function used by [before hook](BeforeHook).
pub(crate) type BeforeFun<D> = for<'a> fn(&'a SlashContext<'a, D>, &'a str) -> BoxFuture<'a, bool>;
/// A hook executed before command execution.
pub struct BeforeHook<D>(pub BeforeFun<D>);

/// A pointer to a function used by [after hook](AfterHook).
pub(crate) type AfterFun<D> =
    for<'a> fn(&'a SlashContext<'a, D>, &'a str, CommandResult) -> BoxFuture<'a, ()>;
/// A hook executed after command execution.
pub struct AfterHook<D>(pub AfterFun<D>);

/// A pointer to a function used by [autocomplete hook](AutocompleteHook)
pub(crate) type AutocompleteFun<D> =
    for<'a> fn(AutocompleteContext<'a, D>) -> BoxFuture<'a, Option<InteractionResponseData>>;
/// A hook used to suggest inputs to the command caller.
pub struct AutocompleteHook<D>(pub AutocompleteFun<D>);
