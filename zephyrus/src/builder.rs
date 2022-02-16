use crate::{
    command::{Command, CommandMap},
    group::{GroupParentBuilder, ParentGroupMap},
    hook::{AfterHook, BeforeHook},
    twilight_exports::{ApplicationMarker, Client, Id},
};
#[cfg(feature = "rc")]
use std::rc::Rc;
use std::sync::Arc;

/// A wrapper around twilight's http client allowing the user to decide how to provide it to the framework.
pub enum WrappedClient {
    Arc(Arc<Client>),
    #[cfg(feature = "rc")]
    Rc(Rc<Client>),
    Raw(Client),
}

impl WrappedClient {
    pub fn inner(&self) -> &Client {
        match self {
            Self::Arc(c) => &c,
            #[cfg(feature = "rc")]
            Self::Rc(c) => &c,
            Self::Raw(c) => &c,
        }
    }
}

impl From<Client> for WrappedClient {
    fn from(c: Client) -> Self {
        WrappedClient::Raw(c)
    }
}

impl From<Arc<Client>> for WrappedClient {
    fn from(c: Arc<Client>) -> Self {
        WrappedClient::Arc(c)
    }
}

#[cfg(feature = "rc")]
impl From<Rc<Client>> for WrappedClient {
    fn from(c: Rc<Client>) -> Self {
        WrappedClient::Rc(c)
    }
}

/// A pointer to a function returning a generic T type.
pub(crate) type FnPointer<T> = fn() -> T;

/// A builder used to set all options before framework initialization.
pub struct FrameworkBuilder<D> {
    /// The http client used by the framework.
    pub http_client: WrappedClient,
    /// The application id of the client.
    pub application_id: Id<ApplicationMarker>,
    /// Data that will be available to all commands.
    pub data: D,
    /// The actual commands, only the simple ones.
    pub commands: CommandMap<D>,
    /// All groups containing commands.
    pub groups: ParentGroupMap<D>,
    /// A hook executed before any command.
    pub before: Option<BeforeHook<D>>,
    /// A hook executed after command's completion.
    pub after: Option<AfterHook<D>>,
}

impl<D: Sized> FrameworkBuilder<D> {
    /// Creates a new [Builder](self::FrameworkBuilder).
    pub fn new(
        http_client: impl Into<WrappedClient>,
        application_id: Id<ApplicationMarker>,
        data: D,
    ) -> Self {
        Self {
            http_client: http_client.into(),
            application_id,
            data,
            commands: Default::default(),
            groups: Default::default(),
            before: None,
            after: None,
        }
    }

    /// Set the hook that will be executed before commands.
    pub fn before(mut self, fun: FnPointer<BeforeHook<D>>) -> Self {
        self.before = Some(fun());
        self
    }

    /// Set the hook that will be executed after command's completion.
    pub fn after(mut self, fun: FnPointer<AfterHook<D>>) -> Self {
        self.after = Some(fun());
        self
    }

    /// Registers a new command.
    pub fn command(mut self, fun: FnPointer<Command<D>>) -> Self {
        let cmd = fun();
        if self.commands.contains_key(cmd.name) || self.groups.contains_key(cmd.name) {
            panic!("{} already registered", cmd.name);
        }
        self.commands.insert(cmd.name.clone(), cmd);
        self
    }

    /// Registers a new group of commands.
    pub fn group<F>(mut self, fun: F) -> Self
    where
        F: FnOnce(&mut GroupParentBuilder<D>) -> &mut GroupParentBuilder<D>,
    {
        let mut builder = GroupParentBuilder::new();
        fun(&mut builder);
        let group = builder.build();

        if self.commands.contains_key(group.name) || self.groups.contains_key(group.name) {
            panic!("{} already registered", group.name);
        }
        self.groups.insert(group.name, group);

        self
    }

    /// Builds the framework, returning a [Framework](crate::framework::Framework).
    pub fn build(self) -> crate::framework::Framework<D> {
        crate::framework::Framework::from_builder(self)
    }
}
