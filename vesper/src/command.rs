use crate::localizations::{Localizations, LocalizationsProvider};
use crate::prelude::{CreateCommandError, Framework};
use crate::{
    argument::CommandArgument, context::SlashContext, twilight_exports::Permissions, BoxFuture, framework::ProcessResult,
};
use std::collections::HashMap;
use tracing::{debug, info};
use twilight_http::client::InteractionClient;
use twilight_model::id::{marker::GuildMarker, Id};
use crate::hook::{CheckHook, ErrorHandlerHook};
use crate::twilight_exports::{Command as TwilightCommand, CommandType};

/// A pointer to a command function.
pub(crate) type CommandFn<D, T, E> = for<'cx, 'data> fn(&'cx mut SlashContext<'data, D>) -> BoxFuture<'cx, Result<T, E>>;
/// A map of [commands](self::Command).
pub type CommandMap<D, T, E> = HashMap<&'static str, Command<D, T, E>>;

#[doc(hidden)]
#[macro_export]
macro_rules! if_some {
    ($item:expr, |$x:ident| $($tree:tt)*) => {
        if let Some($x) = $item {
            $($tree)*
        }
    };
}

/// Information about the execution state of a command.
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum ExecutionState {
    /// A check had an error.
    CheckErrored,
    /// A check returned `false` and the command didn't execute.
    CheckFailed,
    /// The command finished executing without errors.
    CommandFinished,
    /// The error handler raised an error. 
    CommandErrored,
    /// The `before` hook returned `false` and the command didn't execute.
    BeforeHookFailed
}

/// The location of the output of the command.
#[non_exhaustive]
pub enum OutputLocation<T, E> {
    /// The command was not executed, thus there is not any output.
    NotExecuted,
    /// The output has not been taken by any hook.
    Present(Result<T, E>),
    /// The output has been forwarded to the `after` hook.
    TakenByAfterHook,
    /// The output has been taken by the `error_handler` hook.
    TakenByErrorHandler
}

/// Information about the command execution and it's output.
pub struct ExecutionResult<T, E> {
    /// The execution state of the command.
    pub state: ExecutionState,
    /// The output of the command.
    pub output: OutputLocation<T, E>
}

impl<T, E> From<ExecutionResult<T, E>> for ProcessResult<T, E> {
    fn from(value: ExecutionResult<T, E>) -> Self {
        ProcessResult::CommandExecuted(value)
    }
}

/// A command executed by the framework.
pub struct Command<D, T, E> {
    /// The name of the command.
    pub name: &'static str,
    pub localized_names: Localizations<D, T, E>,
    /// The description of the commands.
    pub description: &'static str,
    pub localized_descriptions: Localizations<D, T, E>,
    pub kind: CommandType,
    /// All the arguments the command requires.
    pub arguments: Vec<CommandArgument<D, T, E>>,
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
            localized_names: Default::default(),
            description: Default::default(),
            localized_descriptions: Default::default(),
            kind: CommandType::ChatInput,
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

    pub fn kind(mut self, kind: CommandType) -> Self {
        self.kind = kind;
        self
    }

    /// Adds an argument to the command.
    pub fn add_argument(mut self, arg: CommandArgument<D, T, E>) -> Self {
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

    pub fn localized_names<I, K, V>(mut self, iterator: I) -> Self 
    where
        I: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: ToString
    {
        self.localized_names
            .extend(iterator.into_iter().map(|(k, v)| (k.to_string(), v.to_string())));
        self
    }

    pub fn localized_names_fn(mut self, fun: LocalizationsProvider<D, T, E>) -> Self {
        self.localized_names.set_provider(fun);
        self
    }

    pub fn localized_descriptions<I, K, V>(mut self, iterator: I) -> Self 
    where
        I: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: ToString
    {
        self.localized_descriptions
            .extend(iterator.into_iter().map(|(k, v)| (k.to_string(), v.to_string())));
        self
    }

    pub fn localized_descriptions_fn(mut self, fun: LocalizationsProvider<D, T, E>) -> Self {
        self.localized_descriptions.set_provider(fun);
        self
    }

    pub async fn run_checks<'cx, 'data: 'cx>(&self, context: &'cx mut SlashContext<'data, D>) -> Result<bool, E> {
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

    async fn create_chat_command(
        &self,
        framework: &Framework<D, T, E>,
        http: &InteractionClient<'_>,
        guild: Option<Id<GuildMarker>>
    ) -> Result<TwilightCommand, CreateCommandError>
    {
        let options = self.arguments.iter()
            .map(|a| a.as_option(framework, self))
            .collect::<Vec<_>>();

        let name_localizations = self.localized_names.get_localizations(framework, &self);
        let description_localizations = self.localized_descriptions.get_localizations(framework, &self);

        let model = if let Some(id) = guild {
            let mut command = http.create_guild_command(id)
                .chat_input(self.name, self.description)
                .command_options(&options)
                .nsfw(self.nsfw);

            if_some!(self.required_permissions, |p| command = command.default_member_permissions(p));
            if_some!(&name_localizations, |n| command = command.name_localizations(n));
            if_some!(&description_localizations, |d| command = command.description_localizations(d));

            command.await?.model().await?
        } else {
            let mut command = http.create_global_command()
                .chat_input(self.name, self.description)
                .command_options(&options)
                .nsfw(self.nsfw)
                .dm_permission(!self.only_guilds);

            if_some!(self.required_permissions, |p| command = command.default_member_permissions(p));
            if_some!(&name_localizations, |n| command = command.name_localizations(n));
            if_some!(&description_localizations, |d| command = command.description_localizations(d));

            command.await?.model().await?
        };

        Ok(model)
    }

    async fn create_user_command(
        &self,
        http: &InteractionClient<'_>,
        guild: Option<Id<GuildMarker>>
    ) -> Result<TwilightCommand, CreateCommandError>
    {
        let model = if let Some(id) = guild {
            let mut command = http.create_guild_command(id)
                .user(self.name)
                .nsfw(self.nsfw);

            if_some!(self.required_permissions, |p| command = command.default_member_permissions(p));

            command.await?.model().await?
        } else {
            let mut command = http.create_global_command()
                .user(self.name)
                .nsfw(self.nsfw)
                .dm_permission(!self.only_guilds);

            if_some!(self.required_permissions, |p| command = command.default_member_permissions(p));

            command.await?.model().await?
        };

        Ok(model)
    }

    async fn create_message_command(
        &self,
        http: &InteractionClient<'_>,
        guild: Option<Id<GuildMarker>>
    ) -> Result<TwilightCommand, CreateCommandError>
    {
        let model = if let Some(id) = guild {
            let mut command = http.create_guild_command(id)
                .message(self.name)
                .nsfw(self.nsfw);

            if_some!(self.required_permissions, |p| command = command.default_member_permissions(p));

            command.await?.model().await?
        } else {
            let mut command = http.create_global_command()
                .message(self.name)
                .nsfw(self.nsfw)
                .dm_permission(!self.only_guilds);

            if_some!(self.required_permissions, |p| command = command.default_member_permissions(p));

            command.await?.model().await?
        };

        Ok(model)
    }

    pub async fn create(
        &self,
        framework: &Framework<D, T, E>,
        http: &InteractionClient<'_>, 
        guild: Option<Id<GuildMarker>>
    ) -> Result<TwilightCommand, CreateCommandError>
    {
        match self.kind {
            CommandType::ChatInput => self.create_chat_command(framework, http, guild).await,
            CommandType::Message => self.create_message_command(http, guild).await,
            CommandType::User => self.create_user_command(http, guild).await,
            _ => panic!("Invalid command type")
        }
    }

    pub async fn execute<'cx, 'data: 'cx>(&self, context: &'cx mut SlashContext<'data, D>) -> ExecutionResult<T, E> {
        let state;
        let location;

        match self.run_checks(context).await {
            Ok(true) => {
                debug!("Executing command [{}]", self.name);
                let output = (self.fun)(context).await;

                match (&self.error_handler, output) {
                    (Some(hook), Err(why)) => {
                        info!("Command [{}] raised an error, using established error handler", self.name);
                        state = ExecutionState::CommandErrored;
                        location = OutputLocation::TakenByErrorHandler;

                        (hook.0)(context, why).await;
                    },
                    (_, Ok(res)) => {
                        debug!("Command [{}] executed successfully", self.name);
                        state = ExecutionState::CommandFinished;
                        location = OutputLocation::Present(Ok(res));
                    },
                    (_, Err(res)) => {
                        info!("Command [{}] raised an error, but no error handler was established", self.name);
                        state = ExecutionState::CommandErrored;
                        location = OutputLocation::Present(Err(res));
                    }
                };
            },
            Err(why) => {
                state = ExecutionState::CheckErrored;
                // If the command has an error handler, execute it, if not, discard the error.
                if let Some(hook) = &self.error_handler {
                    info!("Command [{}] check raised an error, using established error handler", self.name);
                    (hook.0)(context, why).await;
                    location = OutputLocation::TakenByErrorHandler;
                } else {
                    info!("Command [{}] check raised an error, but no error handler was established", self.name);
                    location = OutputLocation::Present(Err(why));
                }
            },
            _ => {
                state = ExecutionState::CheckFailed;
                location = OutputLocation::NotExecuted;
            }
        }

        ExecutionResult {
            state,
            output: location
        }
    }
}
