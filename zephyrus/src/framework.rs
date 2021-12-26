use crate::{
    argument::CommandArgument,
    builder::{FrameworkBuilder, WrappedClient},
    command::{Command, CommandMap},
    context::SlashContext,
    group::{GroupParent, ParentGroupMap, ParentType},
    hook::{AfterHook, BeforeHook},
    twilight_exports::*,
    waiter::WaiterSender,
};
use parking_lot::Mutex;
use tracing::debug;

/// The framework used to dispatch slash commands.
pub struct Framework<D> {
    /// The http client used by the framework.
    http_client: WrappedClient,
    /// Data shared across all command and hook invocations.
    pub data: D,
    /// A map of simple commands.
    commands: CommandMap<D>,
    /// A map of command groups including all children.
    groups: ParentGroupMap<D>,
    /// A hook executed before the command.
    before: Option<BeforeHook<D>>,
    /// A hook executed after command's execution.
    after: Option<AfterHook<D>>,
    /// A vector of waiters corresponding to different commands.
    waiters: Mutex<Vec<WaiterSender>>,
}

impl<D> Framework<D> {
    /// Creates a new [Framework](self::Framework) from the given builder.
    pub(crate) fn from_builder(builder: FrameworkBuilder<D>) -> Self {
        Self {
            http_client: builder.http_client,
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
    pub fn builder(http_client: impl Into<WrappedClient>, data: D) -> FrameworkBuilder<D> {
        FrameworkBuilder::new(http_client, data)
    }

    /// Gets the http client used by the framework.
    pub fn http_client(&self) -> &Client {
        self.http_client.inner()
    }

    /// Processes the given interaction, dispatching commands or waking waiters if necessary.
    pub async fn process(&self, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(cmd) => {
                self.try_execute(*cmd).await
            }
            Interaction::MessageComponent(component) => {
                let mut lock = self.waiters.lock();
                let index = lock.iter().position(|sender| sender.check(&component));

                if let Some(index) = index {
                    let sender = lock.remove(index);
                    sender.send(*component);
                    return;
                }
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
        let context =
            SlashContext::new(&self.http_client(), &self.data, &self.waiters, interaction);

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
        guild_id: GuildId,
    ) -> Result<Vec<TwilightCommand>, Box<dyn std::error::Error + Send + Sync>> {
        let mut commands = Vec::new();

        for (_, cmd) in &self.commands {
            let mut options = Vec::new();

            for i in &cmd.fun_arguments {
                options.push(i.as_option());
            }

            commands.push(
                self.http_client()
                    .create_guild_command(guild_id, cmd.name)?
                    .chat_input(cmd.description)?
                    .command_options(&options)?
                    .exec()
                    .await?
                    .model()
                    .await?,
            );
        }

        for (_, group) in &self.groups {
            commands.push(
                self.http_client()
                    .create_guild_command(guild_id, group.name)?
                    .chat_input(group.description)?
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
                self.http_client()
                    .create_global_command(cmd.name)?
                    .chat_input(cmd.description)?
                    .command_options(&options)?
                    .exec()
                    .await?
                    .model()
                    .await?,
            );
        }

        for (_, group) in &self.groups {
            commands.push(
                self.http_client()
                    .create_global_command(group.name)?
                    .chat_input(group.description)?
                    .command_options(&self.create_group(group))?
                    .exec()
                    .await?
                    .model()
                    .await?,
            );
        }

        Ok(commands)
    }

    fn arg_options(&self, arguments: &Vec<CommandArgument>) -> Vec<CommandOption> {
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
