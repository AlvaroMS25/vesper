use crate::{
    command::CommandMap,
    twilight_exports::Permissions,
};
use std::collections::HashMap;

/// A map of [parent groups](self::GroupParent).
pub type ParentGroupMap<D, T, E> = HashMap<&'static str, GroupParent<D, T, E>>;
/// A map of [command groups](self::CommandGroup).
pub type GroupMap<D, T, E> = HashMap<&'static str, CommandGroup<D, T, E>>;

/// Types a [group parent](self::GroupParent) can be.
pub enum ParentType<D, T, E> {
    /// Simple, the group only has subcommands.
    Simple(CommandMap<D, T, E>),
    /// Group, the group has other groups inside of it.
    Group(GroupMap<D, T, E>),
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

    /// Tries to get the [`group`](self::GroupMap) of the given [parent type](self::ParentType),
    /// returning `Some` if the parent variant is a [`group`](self::ParentType::Group).
    pub fn as_group(&self) -> Option<&GroupMap<D, T, E>> {
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
    /// e.g.: /parent/<subcommand..>
    ///
    /// where `parent` is `name`.
    pub name: &'static str,
    /// The description of the upper command.
    pub description: &'static str,
    /// This parent group child commands.
    pub kind: ParentType<D, T, E>,
    /// The required permissions to execute commands inside this group
    pub required_permissions: Option<Permissions>,
}

/// A group of commands, referred by discord as `SubCommandGroup`.
pub struct CommandGroup<D, T, E> {
    /// The upper command
    ///
    /// e.g.: /parent/command/<subcommand..>/<options..>
    ///
    /// where `command` is `name`.
    pub name: &'static str,
    /// The description of this group.
    pub description: &'static str,
    /// The commands this group has as children.
    pub subcommands: CommandMap<D, T, E>,
}
