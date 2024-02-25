use twilight_http::client::InteractionClient;
use twilight_model::{id::{marker::GuildMarker, Id}, application::command::{CommandOption, CommandOptionType}};

use crate::{
    command::{CommandMap, Command},
    twilight_exports::{Command as TwilightCommand, Permissions}, prelude::{CreateCommandError, Framework},
};
use std::collections::HashMap;

/// A map of [parent groups](self::GroupParent).
pub type GroupParentMap<D, T, E> = HashMap<&'static str, GroupParent<D, T, E>>;
/// A map of [command groups](self::CommandGroup).
pub type CommandGroupMap<D, T, E> = HashMap<&'static str, CommandGroup<D, T, E>>;

/// Types a [group parent](self::GroupParent) can be.
pub enum ParentType<D, T, E> {
    /// Simple, the group only has subcommands.
    Simple(CommandMap<D, T, E>),
    /// Group, the group has other groups inside of it.
    Group(CommandGroupMap<D, T, E>),
}

impl<D, T, E> ParentType<D, T, E> {
    /// Tries to get the [`map`](crate::command::CommandMap) of the given
    /// [parent type](self::ParentType), returning `Some` if the parent variant is
    /// [`simple`](self::ParentType::Simple).
    pub fn as_simple(&self) -> Option<&CommandMap<D, T, E>> {
        match self {
            Self::Simple(map) => Some(map),
            _ => None,
        }
    }

    /// Tries to get the [`group`](self::CommandGroupMap) of the given [parent type](self::ParentType),
    /// returning `Some` if the parent variant is a [`group`](self::ParentType::Group).
    pub fn as_group(&self) -> Option<&CommandGroupMap<D, T, E>> {
        match self {
            Self::Group(group) => Some(group),
            _ => None,
        }
    }
}

/// A parent of a group of sub commands, either a
/// map of [commands](crate::command::Command) referred by discord as `SubCommand`
/// or a map of [groups](self::CommandGroup) referred by discord as `SubCommandGroup`.
pub struct GroupParent<D, T, E> {
    /// The name of the upper command
    ///
    /// e.g.: /parent <subcommand..>
    ///
    /// where `parent` is `name`.
    pub name: &'static str,
    /// The description of the upper command.
    pub description: &'static str,
    /// This parent group child commands.
    pub kind: ParentType<D, T, E>,
    /// The required permissions to execute commands inside this group
    pub required_permissions: Option<Permissions>,
    pub nsfw: bool,
    pub only_guilds: bool
}

/// A group of commands, referred by discord as `SubCommandGroup`.
pub struct CommandGroup<D, T, E> {
    /// The upper command
    ///
    /// e.g.: /parent command <subcommand..> <options..>
    ///
    /// where `command` is `name`.
    pub name: &'static str,
    /// The description of this group.
    pub description: &'static str,
    /// The commands this group has as children.
    pub subcommands: CommandMap<D, T, E>,
}

impl<D, T, E> GroupParent<D, T, E> {
    pub async fn create(
        &self,
        framework: &Framework<D, T, E>,
        http: &InteractionClient<'_>,
        guild: Option<Id<GuildMarker>>
    ) -> Result<TwilightCommand, CreateCommandError>
    {
        let options = self.get_options(framework);

        let model = if let Some(id) = guild {
            let mut command = http.create_guild_command(id)
                .chat_input(self.name, self.description)
                .command_options(&options)
                .nsfw(self.nsfw);

            crate::if_some!(self.required_permissions, |p| command = command.default_member_permissions(p));

            command.await?.model().await?
        } else {
            let mut command = http.create_global_command()
                .chat_input(self.name, self.description)
                .command_options(&options)
                .nsfw(self.nsfw)
                .dm_permission(!self.only_guilds);

            crate::if_some!(self.required_permissions, |p| command = command.default_member_permissions(p));

            command.await?.model().await?
        };

        Ok(model)
    }

    pub fn get_options(&self, f: &Framework<D, T, E>) -> Vec<CommandOption> {
        if let ParentType::Group(groups) = &self.kind {
            let mut subgroups = Vec::new();

            for group in groups.values() {
                let mut subcommands = Vec::new();

                for cmd in group.subcommands.values() {
                    subcommands.push(self.create_subcommand(f, cmd));
                }

                subgroups.push(CommandOption {
                    kind: CommandOptionType::SubCommandGroup,
                    name: group.name.to_string(),
                    description: group.description.to_string(),
                    options: Some(subcommands),
                    autocomplete: None,
                    choices: None,
                    required: None,
                    channel_types: None,
                    description_localizations: None,
                    max_length: None,
                    max_value: None,
                    min_length: None,
                    min_value: None,
                    name_localizations: None,
                });
            }

            subgroups
        } else if let ParentType::Simple(commands) = &self.kind {
            let mut subcommands = Vec::new();
            for sub in commands.values() {
                subcommands.push(self.create_subcommand(f, sub));
            }

            subcommands
        } else {
            unreachable!()
        }
    }

    /// Creates a subcommand at the given scope.
    fn create_subcommand(&self, f: &Framework<D, T, E>, cmd: &Command<D, T, E>) -> CommandOption {
        CommandOption {
            kind: CommandOptionType::SubCommand,
            name: cmd.name.to_string(),
            description: cmd.description.to_string(),
            options: Some(cmd.arguments.iter().map(|a| a.as_option(f, cmd)).collect()),
            autocomplete: None,
            choices: None,
            required: None,
            channel_types: None,
            description_localizations: cmd.localized_descriptions.get_localizations(f, cmd),
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            name_localizations: cmd.localized_names.get_localizations(f, cmd),
        }
    }
}
