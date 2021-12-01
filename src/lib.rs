mod argument;
mod group;
mod parse_impl;

pub mod command;

pub mod builder;
pub use macros;
pub mod context;
pub mod framework;
pub mod hook;
pub mod iter;
pub mod message;
pub mod parse;
pub mod waiter;

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// Useful exports to get started quickly
pub mod prelude {
    pub use crate::{
        builder::FrameworkBuilder,
        command::CommandResult,
        context::SlashContext,
        framework::Framework,
        parse::{Parse, ParseError},
        waiter::WaiterReceiver,
    };
    pub use async_trait::async_trait;
    pub use macros::*;
}

pub mod twilight_exports {
    pub use twilight_http::{
        request::application::interaction::update_original_response::UpdateOriginalResponse, Client,
    };
    pub use twilight_model::{
        application::{
            callback::{CallbackData, InteractionResponse},
            command::{
                BaseCommandOptionData, ChannelCommandOptionData, ChoiceCommandOptionData,
                Command as TwilightCommand, CommandOption, CommandOptionChoice, CommandOptionType,
                NumberCommandOptionData, OptionsCommandOptionData,
            },
            interaction::application_command::{
                ApplicationCommand, CommandData, CommandDataOption, CommandOptionValue,
            },
            interaction::message_component::MessageComponentInteraction,
            interaction::Interaction,
        },
        channel::Message,
        gateway::payload::incoming::InteractionCreate,
        id::{ChannelId, GenericId, GuildId, MessageId, RoleId, UserId},
    };
}
