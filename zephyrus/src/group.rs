use crate::{
    builder::FnPointer,
    command::{Command, CommandMap},
    twilight_exports::Permissions,
};
use std::collections::HashMap;

/// A map of [parent groups](self::GroupParent).
pub type ParentGroupMap<D> = HashMap<&'static str, GroupParent<D>>;
/// A map of [command groups](self::CommandGroup).
pub type GroupMap<D> = HashMap<&'static str, CommandGroup<D>>;

/// Types a [group parent](self::GroupParent) can be.
pub enum ParentType<D> {
    /// Simple, the group only has subcommands.
    Simple(CommandMap<D>),
    /// Group, the group has other groups inside of it.
    Group(GroupMap<D>),
}

impl<D> ParentType<D> {
    /// Tries to get the [`map`](crate::command::CommandMap) of the given
    /// [parent type](self::ParentType), returning `Some` if the parent variant is
    /// [`simple`](self::ParentType::Simple).
    pub fn as_simple(&self) -> Option<&CommandMap<D>> {
        match self {
            Self::Simple(map) => Some(map),
            _ => None,
        }
    }

    /// Tries to get the [`group`](self::GroupMap) of the given [parent type](self::ParentType),
    /// returning `Some` if the parent variant is a [`group`](self::ParentType::Group).
    pub fn as_group(&self) -> Option<&GroupMap<D>> {
        match self {
            Self::Group(group) => Some(group),
            _ => None,
        }
    }
}

/// A parent of a group of sub commands, either a
/// map of [commands](crate::command::Command) referred by discord as `SubCommand`
/// or a map of [groups](self::CommandGroup) referred by discord as `SubCommandGroup`.
pub struct GroupParent<D> {
    /// The name of the upper command
    ///
    /// e.g.: /parent/<subcommand..>
    ///
    /// where `parent` is `name`.
    pub name: &'static str,
    /// The description of the upper command.
    pub description: &'static str,
    /// This parent group child commands.
    pub kind: ParentType<D>,
    /// The required permissions to execute commands inside this group
    pub required_permissions: Option<Permissions>,
}

/// A builder of a [group parent](self::GroupParent), see it for documentation.
pub struct GroupParentBuilder<D> {
    name: Option<&'static str>,
    description: Option<&'static str>,
    kind: ParentType<D>,
    required_permissions: Option<Permissions>,
}

impl<D> GroupParentBuilder<D> {
    /// Creates a new builder.
    pub(crate) fn new() -> Self {
        Self {
            name: None,
            description: None,
            kind: ParentType::Group(Default::default()),
            required_permissions: None,
        }
    }

    /// Sets the name of this parent group.
    pub fn name(&mut self, name: &'static str) -> &mut Self {
        self.name = Some(name);
        self
    }

    /// Sets the description of this parent group.
    pub fn description(&mut self, description: &'static str) -> &mut Self {
        self.description = Some(description);
        self
    }

    pub fn required_permissions(&mut self, permissions: Permissions) -> &mut Self {
        self.required_permissions = Some(permissions);
        self
    }

    /// Sets this parent group as a [group](self::ParentType::Group),
    /// allowing to create subcommand groups inside of it.
    pub fn group<F>(&mut self, fun: F) -> &mut Self
    where
        F: FnOnce(&mut CommandGroupBuilder<D>) -> &mut CommandGroupBuilder<D>,
    {
        let mut builder = CommandGroupBuilder::new();
        fun(&mut builder);
        let built = builder.build();

        if let ParentType::Group(map) = &mut self.kind {
            assert!(!map.contains_key(built.name));
            map.insert(built.name, built);
        } else {
            let mut map = GroupMap::new();
            map.insert(built.name, built);
            self.kind = ParentType::Group(map);
        }
        self
    }

    /// Sets this parent group as [simple](self::ParentType::Simple), only allowing subcommands.
    pub fn command(&mut self, fun: FnPointer<Command<D>>) -> &mut Self {
        let command = fun();
        if let ParentType::Simple(map) = &mut self.kind {
            map.insert(command.name, command);
        } else {
            let mut map = CommandMap::new();
            map.insert(command.name, command);
            self.kind = ParentType::Simple(map);
        }
        self
    }

    /// Builds this parent group, returning an [group parent](self::GroupParent).
    pub fn build(self) -> GroupParent<D> {
        assert!(self.name.is_some() && self.description.is_some());
        GroupParent {
            name: self.name.unwrap(),
            description: self.description.unwrap(),
            kind: self.kind,
            required_permissions: self.required_permissions,
        }
    }
}

/// A group of commands, referred by discord as `SubCommandGroup`.
pub struct CommandGroup<D> {
    /// The upper command
    ///
    /// e.g.: /parent/command/<subcommand..>/<options..>
    ///
    /// where `command` is `name`.
    pub name: &'static str,
    /// The description of this group.
    pub description: &'static str,
    /// The commands this group has as children.
    pub subcommands: CommandMap<D>,
}

/// A builder for a [CommandGroup](self::CommandGroup), see it for documentation.
pub struct CommandGroupBuilder<D> {
    name: Option<&'static str>,
    description: Option<&'static str>,
    subcommands: CommandMap<D>,
}

impl<D> CommandGroupBuilder<D> {
    /// Sets the upper command of this group.
    pub fn name(&mut self, name: &'static str) -> &mut Self {
        self.name = Some(name);
        self
    }

    /// Sets the description of this group.
    pub fn description(&mut self, description: &'static str) -> &mut Self {
        self.description = Some(description);
        self
    }

    /// Adds a command to this group.
    pub fn command(&mut self, fun: FnPointer<Command<D>>) -> &mut Self {
        let command = fun();
        self.subcommands.insert(command.name, command);
        self
    }

    /// Builds the builder into a [group](self::CommandGroup).
    pub(crate) fn build(self) -> CommandGroup<D> {
        assert!(self.name.is_some() && self.description.is_some());

        CommandGroup {
            name: self.name.unwrap(),
            description: self.description.unwrap(),
            subcommands: self.subcommands,
        }
    }

    /// Creates a new builder.
    pub(crate) fn new() -> Self {
        Self {
            name: None,
            description: None,
            subcommands: Default::default(),
        }
    }
}
