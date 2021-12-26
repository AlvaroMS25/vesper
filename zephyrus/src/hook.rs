use crate::{command::CommandResult, context::SlashContext, BoxFuture};

/// A pointer to a function used by [before hook](self::BeforeHook).
pub(crate) type BeforeFun<D> = for<'a> fn(&'a SlashContext<D>, &'a str) -> BoxFuture<'a, bool>;
/// A hook executed before command execution.
pub struct BeforeHook<D>(pub BeforeFun<D>);

/// A pointer to a function used by [after hook](self::AfterHook).
pub(crate) type AfterFun<D> = for<'a> fn(&'a SlashContext<D>, &'a str, CommandResult) -> BoxFuture<'a, ()>;
/// A hook executed after command execution.
pub struct AfterHook<D>(pub AfterFun<D>);
