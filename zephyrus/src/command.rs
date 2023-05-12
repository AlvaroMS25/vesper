use crate::{
    argument::CommandArgument, context::SlashContext, twilight_exports::Permissions, BoxFuture,
};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use twilight_http::request::application::command::create_global_command::CreateGlobalChatInputCommand;
use crate::hook::{CheckHook, ErrorHandlerHook};

/// A pointer to a command function.
pub(crate) type CommandFn<D, T, E> = for<'a> fn(&'a SlashContext<'a, D>) -> BoxFuture<'a, Result<T, E>>;
/// A map of [commands](self::Command).
pub type CommandMap<D, T, E> = HashMap<&'static str, Command<D, T, E>>;

pub enum ExecutionResult<T, E> {
    CheckErrored,
    CheckFailed,
    Finished(Option<Result<T, E>>)
}

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
    pub nsfw: bool,
    pub only_guilds: bool,
    pub checks: Vec<CheckHook<D, E>>,
    pub error_handler: Option<ErrorHandlerHook<D, E>>
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
            nsfw: false,
            only_guilds: false,
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

    pub fn error_handler(mut self, hook: ErrorHandlerHook<D, E>) -> Self {
        self.error_handler = Some(hook);
        self
    }

    pub fn required_permissions(mut self, permissions: Permissions) -> Self {
        self.required_permissions = Some(permissions);
        self
    }

    pub fn nsfw(mut self, nsfw: bool) -> Self {
        self.nsfw = nsfw;
        self
    }

    pub fn only_guilds(mut self, only_guilds: bool) -> Self {
        self.only_guilds = only_guilds;
        self
    }

    pub async fn run_checks(&self, context: &SlashContext<'_, D>) -> Result<bool, E> {
        debug!("Running command [{}] checks", self.name);
        for check in &self.checks {
            if !(check.0)(context).await? {
                debug!("Command [{}] check returned false", self.name);
                return Ok(false);
            }
        }
        debug!("All command [{}] checks passed", self.name);
        Ok(true)
    }

    pub fn apply_global<'a>(
        &self, 
        command: CreateGlobalChatInputCommand<'a>
    ) -> CreateGlobalChatInputCommand<'a> {
        command
            .nsfw(self.nsfw)
            .dm_permission(!self.only_guilds)
    }

    pub async fn execute(&self, context: &SlashContext<'_, D>) -> ExecutionResult<T, E> {
        match self.run_checks(context).await {
            Ok(true) => {
                debug!("Executing command [{}]", self.name);
                let output = (self.fun)(context).await;

                let remainder = match (&self.error_handler, output) {
                    (Some(hook), Err(why)) => {
                        info!("Command [{}] raised an error, using established error handler", self.name);
                        (hook.0)(context, why).await;
                        None
                    },
                    (_, Ok(res)) => {
                        debug!("Command [{}] executed successfully", self.name);
                        Some(Ok(res))
                    },
                    (_, Err(res)) => {
                        info!("Command [{}] raised an error, but no error handler was established,\
                        the error will be forwarded to the after handler", self.name);
                        Some(Err(res))
                    }
                };

                ExecutionResult::Finished(remainder)
            },
            Err(why) => {
                // If the command has an error handler, execute it, if not, discard the error.
                if let Some(hook) = &self.error_handler {
                    info!("Command [{}] check raised an error, using established error handler", self.name);
                    (hook.0)(context, why).await;
                } else {
                    warn!("Command [{}] check raised an error, but no error handler was established", self.name);
                }
                ExecutionResult::CheckErrored
            },
            _ => ExecutionResult::CheckFailed
        }
    }
}
