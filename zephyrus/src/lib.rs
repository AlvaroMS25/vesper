mod parse_impl;

pub mod argument;
pub mod builder;
pub mod command;
pub mod context;
pub mod framework;
pub mod group;
pub mod hook;
pub mod iter;
pub mod parse;
pub mod range;
mod waiter;

pub use zephyrus_macros as macros;

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// Useful exports to get started quickly
pub mod prelude {
    pub use crate::{
        argument::ArgumentLimits,
        builder::{FrameworkBuilder, WrappedClient},
        command::CommandResult,
        context::{AutocompleteContext, Focused, SlashContext},
        framework::Framework,
        parse::{Parse, ParseError},
        range::Range,
    };
    pub use async_trait::async_trait;
    pub use zephyrus_macros::*;
}

pub mod twilight_exports {
    pub use twilight_http::{
        client::{Client, InteractionClient},
        request::application::interaction::UpdateResponse,
    };
    pub use twilight_model::{
        application::{
            command::{
                BaseCommandOptionData, ChannelCommandOptionData, ChoiceCommandOptionData, Command,
                CommandOption, CommandOptionChoice, CommandOptionType, NumberCommandOptionData,
                OptionsCommandOptionData,
            },
            interaction::{
                application_command::{
                    CommandData, CommandDataOption, CommandOptionValue,
                },
                message_component::MessageComponentInteractionData,
                Interaction,
                InteractionType,
                InteractionData
            },
        },
        channel::Message,
        gateway::payload::incoming::InteractionCreate,
        guild::Permissions,
        http::interaction::{
            InteractionResponse, InteractionResponseData, InteractionResponseType,
        },
        id::{
            marker::{
                ApplicationMarker, ChannelMarker, GenericMarker, GuildMarker, MessageMarker,
                RoleMarker, UserMarker,
            },
            Id,
        },
    };
}
