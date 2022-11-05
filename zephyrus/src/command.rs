use crate::{
    argument::CommandArgument, context::SlashContext, twilight_exports::Permissions, BoxFuture,
};
use std::collections::HashMap;
use crate::hook::{CheckHook, ErrorHandlerHook};

/// A pointer to a command function.
pub(crate) type CommandFn<D, T, E> = for<'a> fn(&'a SlashContext<'a, D>) -> BoxFuture<'a, Result<T, E>>;
/// A map of [commands](self::Command).
pub type CommandMap<D, T, E> = HashMap<&'static str, Command<D, T, E>>;

/// A command executed by the framework.
pub struct Command<D, T, E> {
    /// The name of the command.
    pub name: &'static str,
    /// The description of the commands.
    pub description: &'static str,
    /// All the arguments the command requires.
    pub arguments: Vec<CommandArgument<D>>,
    /// A pointer to this command function.
    pub fun: CommandFn<D, T, E>,
    /// The required permissions to use this command
    pub required_permissions: Option<Permissions>,
    pub checks: Vec<CheckHook<D, E>>,
    pub error_handler: Option<ErrorHandlerHook<D, T, E>>
}

impl<D, T, E> Command<D, T, E> {
    /// Creates a new command.
    pub fn new(fun: CommandFn<D, T, E>) -> Self {
        Self {
            name: Default::default(),
            description: Default::default(),
            arguments: Default::default(),
            fun,
            required_permissions: Default::default(),
            checks: Default::default(),
            error_handler: None
        }
    }

    /// Sets the command name.
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Sets the command description.
    pub fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    /// Adds an argument to the command.
    pub fn add_argument(mut self, arg: CommandArgument<D>) -> Self {
        self.arguments.push(arg);
        self
    }

    pub fn checks(mut self, checks: Vec<CheckHook<D, E>>) -> Self {
        self.checks = checks;
        self
    }

    pub fn error_handler(mut self, hook: ErrorHandlerHook<D, T, E>) -> Self {
        self.error_handler = Some(hook);
        self
    }

    pub fn required_permissions(mut self, permissions: Permissions) -> Self {
        self.required_permissions = Some(permissions);
        self
    }

    pub async fn run_checks(&self, context: &SlashContext<'_, D>) -> Result<bool, E> {
        for check in &self.checks {
            if !(check.0)(context).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub async fn execute(&self, context: &SlashContext<'_, D>) -> Option<Result<T, E>> {
        match self.run_checks(context).await {
            Ok(true) => {
                let output = (self.fun)(context).await;
                if let Some(hook) = &self.error_handler {
                    (hook.0)(context, output).await;

                    None
                } else {
                    Some(output)
                }
            },
            Err(why) => Some(Err(why)),
            _ => None
        }
    }
}
