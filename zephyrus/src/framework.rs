use crate::{
    argument::CommandArgument,
    builder::{FrameworkBuilder, WrappedClient},
    command::{Command, CommandMap},
    context::SlashContext,
    group::{GroupParent, ParentGroupMap, ParentType},
    hook::{AfterHook, BeforeHook},
    twilight_exports::{
        ApplicationCommand, ApplicationCommandAutocomplete, ApplicationMarker, Client,
        Command as TwilightCommand, CommandDataOption, CommandOption, CommandOptionType,
        CommandOptionValue, GuildMarker, Id, Interaction, InteractionClient,
        OptionsCommandOptionData,
    },
    waiter::WaiterSender,
};
use parking_lot::Mutex;
use std::future::Future;
use tracing::debug;
use twilight_model::application::interaction::application_command_autocomplete::ApplicationCommandAutocompleteDataOptionType;

/// The framework used to dispatch slash commands.
pub struct Framework<D> {
    /// The http client used by the framework.
    pub http_client: WrappedClient,
    /// The application id of the client.
    pub application_id: Id<ApplicationMarker>,
    /// Data shared across all command and hook invocations.
    pub data: D,
    /// A map of simple commands.
    pub commands: CommandMap<D>,
    /// A map of command groups including all children.
    pub groups: ParentGroupMap<D>,
    /// A hook executed before the command.
    pub before: Option<BeforeHook<D>>,
    /// A hook executed after command's execution.
    pub after: Option<AfterHook<D>>,
    /// A vector of waiters corresponding to different commands.
    waiters: Mutex<Vec<WaiterSender>>,
}

impl<D> Framework<D> {
    /// Creates a new [Framework](self::Framework) from the given builder.
    pub(crate) fn from_builder(builder: FrameworkBuilder<D>) -> Self {
        Self {
            http_client: builder.http_client,
            application_id: builder.application_id,
            data: builder.data,
            commands: builder.commands,
            groups: builder.groups,
            before: builder.before,
            after: builder.after,
            waiters: Mutex::new(Vec::new()),
        }
    }

    /// Creates a new framework builder, this is a shortcut to FrameworkBuilder.
    /// [new](crate::builder::FrameworkBuilder::new)
    pub fn builder(
        http_client: impl Into<WrappedClient>,
        application_id: Id<ApplicationMarker>,
        data: D,
    ) -> FrameworkBuilder<D> {
        FrameworkBuilder::new(http_client, application_id, data)
    }

    /// Gets the http client used by the framework.
    pub fn http_client(&self) -> &Client {
        self.http_client.inner()
    }

    /// Gets the [interaction client](InteractionClient) using this framework's
    /// [http client](Client) and [application id](ApplicationMarker)
    pub fn interaction_client(&self) -> InteractionClient {
        self.http_client().interaction(self.application_id)
    }

    /// Processes the given interaction, dispatching commands or waking waiters if necessary.
    pub async fn process(&self, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(cmd) => self.try_execute(*cmd).await,
            Interaction::MessageComponent(component) => {
                let mut lock = self.waiters.lock();
                let index = lock.iter().position(|sender| sender.check(&component));

                if let Some(index) = index {
                    let sender = lock.remove(index);
                    sender.send(*component);
                    return;
                }
            }
            Interaction::ApplicationCommandAutocomplete(autocomplete) => {
                self.try_autocomplete(*autocomplete).await
            }
            _ => return,
        }
    }

    /// Tries to execute a command based on the given
    /// [ApplicationCommand](ApplicationCommand).
    async fn try_execute(&self, mut interaction: ApplicationCommand) {
        if let Some(command) = self.get_command(&mut interaction) {
            self.execute(command, interaction).await;
        }
    }

    async fn try_autocomplete(&self, autocomplete: ApplicationCommandAutocomplete) {
        println!("{:?}", autocomplete);

        if let Some(future) = self.get_autocomplete(autocomplete) {
            future.await;
        }
    }

    fn get_autocomplete<'a>(
        &'a self,
        mut autocomplete: ApplicationCommandAutocomplete,
    ) -> Option<std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'a>>> {
        if autocomplete.data.options.len() > 0 {
            let mut inner = autocomplete.data.options.remove(0);
            match inner.kind {
                ApplicationCommandAutocompleteDataOptionType::SubCommandGroup => {
                    if inner.options.len() > 0 {
                        let map = self
                            .groups
                            .get(autocomplete.data.name.as_str())?
                            .kind
                            .as_group()?;
                        let group = map.get(inner.name.as_str())?;
                        let mut inner = inner.options.remove(0);
                        if let ApplicationCommandAutocompleteDataOptionType::SubCommand = inner.kind
                        {
                            if inner.options.len() > 0 {
                                let command = group.subcommands.get(inner.name.as_str())?;
                                let inner = inner.options.remove(0);
                                let position = command
                                    .fun_arguments
                                    .iter()
                                    .position(|arg| arg.name == &inner.name)?;
                                let arg = command.fun_arguments.get(position)?;
                                return Some((arg.autocomplete.as_ref()?.0)(
                                    &self.http_client,
                                    &self.data,
                                    inner.value,
                                ));
                            }
                        }
                    }
                }
                ApplicationCommandAutocompleteDataOptionType::SubCommand => {
                    if inner.options.len() > 0 {
                        let subcommands = self
                            .groups
                            .get(autocomplete.data.name.as_str())?
                            .kind
                            .as_simple()?;
                        let command = subcommands.get(inner.name.as_str())?;
                        let inner = inner.options.remove(0);
                        let position = command
                            .fun_arguments
                            .iter()
                            .position(|arg| arg.name == &inner.name)?;
                        let arg = command.fun_arguments.get(position)?;
                        return Some((arg.autocomplete.as_ref()?.0)(
                            &self.http_client,
                            &self.data,
                            inner.value,
                        ));
                    }
                }
                _ => {
                    let command = self.commands.get(autocomplete.data.name.as_str())?;
                    let position = command
                        .fun_arguments
                        .iter()
                        .position(|arg| arg.name == &inner.name)?;
                    let arg = command.fun_arguments.get(position)?;
                    return Some((arg.autocomplete.as_ref()?.0)(
                        &self.http_client,
                        &self.data,
                        inner.value,
                    ));
                }
            }
        }

        None
    }

    /// Gets the command matching the given
    /// [ApplicationCommand](ApplicationCommand),
    /// returning `None` if no command matches the given interaction.
    fn get_command(&self, interaction: &mut ApplicationCommand) -> Option<&Command<D>> {
        if let Some(next) = self.get_next(&mut interaction.data.options) {
            let group = self.groups.get(&*interaction.data.name)?;
            match next.value.kind() {
                CommandOptionType::SubCommand => {
                    let subcommands = group.kind.as_simple()?;
                    let options = match next.value {
                        CommandOptionValue::SubCommand(s) => s,
                        _ => unreachable!(),
                    };
                    interaction.data.options = options;
                    subcommands.get(&*next.name)
                }
                CommandOptionType::SubCommandGroup => {
                    let mut options = match next.value {
                        CommandOptionValue::SubCommandGroup(s) => s,
                        _ => unreachable!(),
                    };
                    let subcommand = self.get_next(&mut options)?;
                    let subgroups = group.kind.as_group()?;
                    let group = subgroups.get(&*next.name)?;
                    let options = match subcommand.value {
                        CommandOptionValue::SubCommand(s) => s,
                        _ => unreachable!(),
                    };
                    interaction.data.options = options;
                    group.subcommands.get(&*subcommand.name)
                }
                _ => None,
            }
        } else {
            self.commands.get(&*interaction.data.name)
        }
    }

    /// Gets the next [option](CommandDataOption)
    /// only if it corresponds to a subcommand or a subcommand group.
    fn get_next(&self, interaction: &mut Vec<CommandDataOption>) -> Option<CommandDataOption> {
        if interaction.len() > 0
            && (interaction[0].value.kind() == CommandOptionType::SubCommand
                || interaction[0].value.kind() == CommandOptionType::SubCommandGroup)
        {
            Some(interaction.remove(0))
        } else {
            None
        }
    }

    /// Executes the given [command](crate::command::Command) and the hooks.
    async fn execute(&self, cmd: &Command<D>, interaction: ApplicationCommand) {
        let context = SlashContext::new(
            &self.http_client,
            self.application_id,
            &self.data,
            &self.waiters,
            interaction,
        );

        let execute = if let Some(before) = &self.before {
            (before.0)(&context, cmd.name).await
        } else {
            true
        };

        if execute {
            let result = (cmd.fun)(&context).await;

            if let Some(after) = &self.after {
                (after.0)(&context, cmd.name, result).await;
            }
        }
    }

    pub async fn register_guild_commands(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Vec<TwilightCommand>, Box<dyn std::error::Error + Send + Sync>> {
        let mut commands = Vec::new();

        for (_, cmd) in &self.commands {
            let mut options = Vec::new();

            for i in &cmd.fun_arguments {
                options.push(i.as_option());
            }

            commands.push(
                self.interaction_client()
                    .create_guild_command(guild_id)
                    .chat_input(cmd.name, cmd.description)?
                    .command_options(&options)?
                    .exec()
                    .await?
                    .model()
                    .await?,
            );
        }

        for (_, group) in &self.groups {
            commands.push(
                self.interaction_client()
                    .create_guild_command(guild_id)
                    .chat_input(group.name, group.description)?
                    .command_options(&self.create_group(group))?
                    .exec()
                    .await?
                    .model()
                    .await?,
            );
        }

        Ok(commands)
    }

    pub async fn register_global_commands(
        &self,
    ) -> Result<Vec<TwilightCommand>, Box<dyn std::error::Error + Send + Sync>> {
        let mut commands = Vec::new();

        for (_, cmd) in &self.commands {
            let mut options = Vec::new();

            for i in &cmd.fun_arguments {
                options.push(i.as_option());
            }

            commands.push(
                self.interaction_client()
                    .create_global_command()
                    .chat_input(cmd.name, cmd.description)?
                    .command_options(&options)?
                    .exec()
                    .await?
                    .model()
                    .await?,
            );
        }

        for (_, group) in &self.groups {
            commands.push(
                self.interaction_client()
                    .create_global_command()
                    .chat_input(group.name, group.description)?
                    .command_options(&self.create_group(group))?
                    .exec()
                    .await?
                    .model()
                    .await?,
            );
        }

        Ok(commands)
    }

    fn arg_options(&self, arguments: &Vec<CommandArgument<D>>) -> Vec<CommandOption> {
        let mut options = Vec::with_capacity(arguments.len());

        for arg in arguments {
            options.push(arg.as_option());
        }

        options
    }

    fn create_group(&self, parent: &GroupParent<D>) -> Vec<CommandOption> {
        debug!("Registering group {}", parent.name);

        if let ParentType::Group(map) = &parent.kind {
            let mut subgroups = Vec::new();
            for (_, group) in map {
                debug!("Registering subgroup {} of {}", group.name, parent.name);

                let mut subcommands = Vec::new();
                for (_, sub) in &group.subcommands {
                    subcommands.push(self.create_subcommand(sub))
                }

                subgroups.push(CommandOption::SubCommandGroup(OptionsCommandOptionData {
                    name: group.name.to_string(),
                    description: group.description.to_string(),
                    options: subcommands,
                }));
            }
            subgroups
        } else if let ParentType::Simple(map) = &parent.kind {
            let mut subcommands = Vec::new();
            for (_, sub) in map {
                subcommands.push(self.create_subcommand(sub));
            }

            subcommands
        } else {
            unreachable!()
        }
    }

    /// Creates a subcommand at the given scope.
    fn create_subcommand(&self, cmd: &Command<D>) -> CommandOption {
        debug!("Registering {} subcommand", cmd.name);

        CommandOption::SubCommand(OptionsCommandOptionData {
            name: cmd.name.to_string(),
            description: cmd.description.to_string(),
            options: self.arg_options(&cmd.fun_arguments),
        })
    }
}
