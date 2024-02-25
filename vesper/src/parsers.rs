use std::ops::{Deref, DerefMut};
use async_trait::async_trait;
use twilight_model::application::command::{CommandOption, CommandOptionType};
use twilight_model::application::interaction::{InteractionDataResolved, InteractionChannel};
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::channel::ChannelType;
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;
use crate::builder::WrappedClient;
use crate::parse::{Parse, ParseError};
use crate::parse_impl::error;

macro_rules! newtype_struct {
    ($($(#[$meta:meta])* $v: vis struct $name: ident($inner: ty)),* $(,)?) => {
        $(
            newtype_struct!(@inner $(#[$meta])* $v struct $name($inner));
        )*
    };
    (@inner $(#[$meta:meta])* $v: vis struct $name: ident($inner: ty)) => {
        $(#[$meta])*
        $v struct $name($inner);

        impl Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    }
}

macro_rules! parse_id {
    ($($name: ty, $kind: expr, [$($allowed: expr),* $(,)?]),* $(,)?) => {
        $(
            parse_id!(@inner $name, $kind, [$($allowed),*]);
        )*
    };
    (@inner $name: ty, $kind: expr, [$($allowed: expr),* $(,)?]) => {
        #[async_trait]
        impl<T: Send + Sync> Parse<T> for $name {
            async fn parse(
                http_client: &WrappedClient,
                data: &T,
                value: Option<&CommandOptionValue>,
                resolved: Option<&mut InteractionDataResolved>
            ) -> Result<Self, ParseError> {
                Ok(Self(Id::parse(http_client, data, value, resolved).await?))
            }

            fn kind() -> CommandOptionType {
                $kind
            }

            fn modify_option(option: &mut CommandOption) {
                option.channel_types = Some(vec![$($allowed),*])
            }
        }
    };
}

macro_rules! parse_derived_channel {
    ($($name_t: ty, $id: ty, $name: literal),* $(,)?) => {
        $(
            parse_derived_channel!(@inner $name_t, $id, $name);
        )*
    };
    (@inner $name_t: ty, $id: ty, $name: literal) => {
        #[async_trait]
        impl<T: Send + Sync> Parse<T> for $name_t {
            async fn parse(
                http_client: &WrappedClient,
                data: &T,
                value: Option<&CommandOptionValue>,
                resolved: Option<&mut InteractionDataResolved>
            ) -> Result<Self, ParseError> {
                let id = <$id>::parse(http_client, data, value, None).await?;

                resolved.map(|items| items.channels.remove(&*id))
                    .flatten()
                    .ok_or_else(|| error($name, true, concat!($name, " expected")))
                    .map(Self)
            }

            fn kind() -> CommandOptionType {
                <$id as Parse<T>>::kind()
            }

            fn modify_option(option: &mut CommandOption) {
                <$id as Parse<T>>::modify_option(option)
            }
        }
    };
}

newtype_struct! {
    /// An object that parses into a discord only **text** channel.
    pub struct TextChannel(InteractionChannel),
    /// An object that parses into a discord only **text** channel id.
    pub struct TextChannelId(Id<ChannelMarker>),
    /// An object that parses into a discord only **voice** channel.
    pub struct VoiceChannel(InteractionChannel),
    /// An object that parses into a discord only **voice** channel id.
    pub struct VoiceChannelId(Id<ChannelMarker>),
    /// An object that parses into a discord **only public** thread.
    pub struct PublicThread(InteractionChannel),
    /// An object that parses into a discord **only public** thread id.
    pub struct PublicThreadId(Id<ChannelMarker>),
    /// An object that parses into a discord **only private** thread.
    pub struct PrivateThread(InteractionChannel),
    /// An object that parses into a discord **only private** thread id.
    pub struct PrivateThreadId(Id<ChannelMarker>),
    /// An object that parses into a discord **either public, private or announcement** thread.
    pub struct Thread(InteractionChannel),
    /// An object that parses into a discord **either public, private or announcement** thread id.
    pub struct ThreadId(Id<ChannelMarker>)
}

parse_id! {
    TextChannelId, CommandOptionType::Channel, [ChannelType::GuildText],
    VoiceChannelId, CommandOptionType::Channel, [ChannelType::GuildVoice],
    PublicThreadId, CommandOptionType::Channel, [ChannelType::PublicThread],
    PrivateThreadId, CommandOptionType::Channel, [ChannelType::PrivateThread],
    ThreadId, CommandOptionType::Channel, [ChannelType::PublicThread, ChannelType::PrivateThread],
}

parse_derived_channel! {
    TextChannel, TextChannelId, "Text Channel",
    VoiceChannel, VoiceChannelId, "Voice Channel",
    PublicThread, PublicThreadId, "Public Thread",
    PrivateThread, PrivateThreadId, "Private Thread",
    Thread, ThreadId, "Thread"
}
