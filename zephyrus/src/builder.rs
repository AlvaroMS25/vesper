use crate::{
    command::{Command, CommandMap},
    framework::{DefaultError, Framework},
    group::*,
    hook::{AfterHook, BeforeHook},
    twilight_exports::{ApplicationMarker, Client, Id, Permissions},
    parse::ParseError
};
#[cfg(feature = "rc")]
use std::rc::Rc;
use std::{ops::Deref, sync::Arc};

/// A wrapper around twilight's http client allowing the user to decide how to provide it to the framework.
#[allow(clippy::large_enum_variant)]
pub enum WrappedClient {
    Arc(Arc<Client>),
    #[cfg(feature = "rc")]
    Rc(Rc<Client>),
    Raw(Client),
    Boxed(Box<dyn Deref<Target = Client> + Send + Sync>),
}

impl WrappedClient {
    /// Returns the underlying http client.
    pub fn inner(&self) -> &Client {
        match self {
            Self::Arc(c) => c,
            #[cfg(feature = "rc")]
            Self::Rc(c) => &c,
            Self::Raw(c) => c,
            Self::Boxed(b) => b,
        }
    }

    /// Casts the [client](WrappedClient) into T if it's [Boxed](WrappedClient::Boxed)
    ///
    /// **SAFETY: The caller must ensure the type given is the same as the boxed one.**
    #[allow(clippy::needless_lifetimes, clippy::borrow_deref_ref)]
    pub fn cast<'a, T>(&'a self) -> Option<&'a T> {
        if let WrappedClient::Boxed(inner) = self {
            // SAFETY: The caller must ensure here that the type provided is the original type of
            // the pointer.
            let ptr = (&*inner.as_ref()) as *const _ as *const T;
            // SAFETY: It is safe to dereference here the pointer as we hold the owned value,
            // so we ensure it is valid.
            Some(unsafe { &*ptr })
        } else {
            None
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

impl From<Box<dyn Deref<Target = Client> + Send + Sync>> for WrappedClient {
    fn from(c: Box<dyn Deref<Target = Client> + Send + Sync>) -> Self {
        Self::Boxed(c)
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
pub struct FrameworkBuilder<D, T = (), E = DefaultError> {
    /// The http client used by the framework.
    pub http_client: WrappedClient,
    /// The application id of the client.
    pub application_id: Id<ApplicationMarker>,
    /// Data that will be available to all commands.
    pub data: D,
    /// The actual commands, only the simple ones.
    pub commands: CommandMap<D, T, E>,
    /// All groups containing commands.
    pub groups: GroupParentMap<D, T, E>,
    /// A hook executed before any command.
    pub before: Option<BeforeHook<D>>,
    /// A hook executed after command's completion.
    pub after: Option<AfterHook<D, T, E>>,
}

impl<D, T, E> FrameworkBuilder<D, T, E>
where
    E: From<ParseError>
{
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zephyrus::prelude::*;
    /// use twilight_http::Client;
    /// use twilight_model::id::Id;
    ///
    /// #[before]
    /// async fn before_hook(ctx: &SlashContext<()>, command_name: &str) -> bool {
    ///     println!("Executing command {command_name}");
    ///     true
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let token = std::env::var("DISCORD_TOKEN").unwrap();
    ///     let app_id = std::env::var("DISCORD_APP_ID").unwrap().parse::<u64>().unwrap();
    ///     let http_client = Client::new(token);
    ///
    ///     let framework = Framework::<()>::builder(http_client, Id::new(app_id), ())
    ///         .before(before_hook)
    ///         .build();
    /// }
    /// ```
    pub fn before(mut self, fun: FnPointer<BeforeHook<D>>) -> Self {
        self.before = Some(fun());
        self
    }

    /// Set the hook that will be executed after command's completion.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zephyrus::prelude::*;
    /// use twilight_http::Client;
    /// use twilight_model::id::Id;
    ///
    /// #[after]
    /// async fn after_hook(ctx: &SlashContext<()>, command_name: &str, _: Option<DefaultCommandResult>) {
    ///     println!("Command {command_name} finished execution");
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let token = std::env::var("DISCORD_TOKEN").unwrap();
    ///     let app_id = std::env::var("DISCORD_APP_ID").unwrap().parse::<u64>().unwrap();
    ///     let http_client = Client::new(token);
    ///
    ///     let framework = Framework::builder(http_client, Id::new(app_id), ())
    ///         .after(after_hook)
    ///         .build();
    /// }
    /// ```
    pub fn after(mut self, fun: FnPointer<AfterHook<D, T, E>>) -> Self {
        self.after = Some(fun());
        self
    }

    /// Registers a new command in the framework.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zephyrus::prelude::*;
    /// use twilight_http::Client;
    /// use twilight_model::id::Id;
    ///
    ///#[command]
    ///#[description = "Says Hello world!"]
    ///async fn hello_world(ctx: &SlashContext<()>) -> DefaultCommandResult {
    ///     ctx.defer(false).await?;
    ///     ctx.interaction_client.update_response(&ctx.interaction.token)
    ///         .content(Some("Hello world!"))
    ///         .unwrap()
    ///         .await?;
    ///
    ///     Ok(())
    ///}
    ///
    ///#[command]
    ///#[description = "Repeats something"]
    ///async fn repeat_content(
    ///     ctx: &SlashContext<()>,
    ///     #[rename = "content"] #[description = "The content"] c: String
    ///) -> DefaultCommandResult
    ///{
    ///     ctx.defer(false).await?;
    ///     ctx.interaction_client.update_response(&ctx.interaction.token)
    ///         .content(Some(&c))
    ///         .unwrap()
    ///         .await?;
    ///     Ok(())
    ///}
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let token = std::env::var("DISCORD_TOKEN").unwrap();
    ///     let app_id = std::env::var("DISCORD_APP_ID").unwrap().parse::<u64>().unwrap();
    ///     let http_client = Client::new(token.clone());
    ///
    ///     let framework = Framework::builder(http_client, Id::new(app_id), ())
    ///         .command(hello_world)
    ///         .command(repeat_content)
    ///         .build();
    /// }
    /// ```
    pub fn command(mut self, fun: FnPointer<Command<D, T, E>>) -> Self {
        let cmd = fun();
        if self.commands.contains_key(cmd.name) || self.groups.contains_key(cmd.name) {
            panic!("{} already registered", cmd.name);
        }
        self.commands.insert(cmd.name, cmd);
        self
    }

    /// Registers a new group of commands.
    pub fn group<F>(mut self, fun: F) -> Self
    where
        F: FnOnce(&mut GroupParentBuilder<D, T, E>) -> &mut GroupParentBuilder<D, T, E>,
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
    pub fn build(self) -> Framework<D, T, E> {
        Framework::from_builder(self)
    }
}

/// A builder of a [group parent](crate::group::GroupParent), see it for documentation.
pub struct GroupParentBuilder<D, T, E> {
    name: Option<&'static str>,
    description: Option<&'static str>,
    kind: ParentType<D, T, E>,
    required_permissions: Option<Permissions>,
    nsfw: bool,
    only_guilds: bool
}

impl<D, T, E> GroupParentBuilder<D, T, E> {
    /// Creates a new builder.
    pub(crate) fn new() -> Self {
        Self {
            name: None,
            description: None,
            kind: ParentType::Group(Default::default()),
            required_permissions: None,
            nsfw: false,
            only_guilds: false
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

    pub fn nsfw(&mut self, nsfw: bool) -> &mut Self {
        self.nsfw = nsfw;
        self
    }

    pub fn only_guilds(&mut self, only_guilds: bool) -> &mut Self {
        self.only_guilds = only_guilds;
        self
    }

    /// Sets this parent group as a [group](crate::group::ParentType::Group),
    /// allowing to create subcommand groups inside of it.
    pub fn group<F>(&mut self, fun: F) -> &mut Self
        where
            F: FnOnce(&mut CommandGroupBuilder<D, T, E>) -> &mut CommandGroupBuilder<D, T, E>,
    {
        let mut builder = CommandGroupBuilder::new();
        fun(&mut builder);
        let built = builder.build();

        if let ParentType::Group(map) = &mut self.kind {
            assert!(!map.contains_key(built.name));
            map.insert(built.name, built);
        } else {
            let mut map = CommandGroupMap::new();
            map.insert(built.name, built);
            self.kind = ParentType::Group(map);
        }
        self
    }

    /// Sets this parent group as [simple](crate::group::ParentType::Simple), only allowing subcommands.
    pub fn command(&mut self, fun: FnPointer<Command<D, T, E>>) -> &mut Self {
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

    /// Builds this parent group, returning a [group parent](crate::group::GroupParent).
    pub fn build(self) -> GroupParent<D, T, E> {
        assert!(self.name.is_some() && self.description.is_some());
        GroupParent {
            name: self.name.unwrap(),
            description: self.description.unwrap(),
            kind: self.kind,
            required_permissions: self.required_permissions,
            nsfw: self.nsfw,
            only_guilds: self.only_guilds
        }
    }
}

/// A builder for a [command group](crate::group::CommandGroup), see it for documentation.
pub struct CommandGroupBuilder<D, T, E> {
    name: Option<&'static str>,
    description: Option<&'static str>,
    subcommands: CommandMap<D, T, E>,
}

impl<D, T, E> CommandGroupBuilder<D, T, E> {
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
    pub fn command(&mut self, fun: FnPointer<Command<D, T, E>>) -> &mut Self {
        let command = fun();
        self.subcommands.insert(command.name, command);
        self
    }

    /// Builds the builder into a [group](crate::group::CommandGroup).
    pub(crate) fn build(self) -> CommandGroup<D, T, E> {
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
