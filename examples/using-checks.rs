use std::env;
use std::sync::Arc;
use tokio::task::JoinSet;
use twilight_gateway::{create_recommended, Config, EventTypeFlags, StreamExt};
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::Intents;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use twilight_model::id::Id;
use vesper::framework::DefaultError;
use vesper::prelude::*;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").unwrap();
    let application_id = env::var("DISCORD_APPLICATION_ID").unwrap().parse::<u64>().unwrap();

    let http_client = Arc::new(Client::new(token.clone()));

    let config = Config::new(token.clone(), Intents::empty());
    let shards = create_recommended(
        &http_client,
        config,
        |_, builder| builder.build()
    ).await.unwrap().collect::<Vec<_>>();

    let framework = Arc::new(Framework::builder(http_client, Id::new(application_id), ())
        .command(hello)
        .build());

    let mut set = JoinSet::new();
    for mut shard in shards {
        let framework = Arc::clone(&framework);
        set.spawn(async move {
            while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
                let Ok(event) = item else {
                    eprintln!("error receiving event: {:?}", item.unwrap_err());
                    continue;
                };
                match event {
                    Event::Ready(_) => {
                        // We have to register the commands for them to show in discord.
                        framework.register_global_commands().await.unwrap();
                    },
                    Event::InteractionCreate(interaction) => {
                        framework.process(interaction.0).await;
                    },
                    _ => ()
                }
            }
        });
    }

    while set.join_next().await.is_some() {}
}

#[check]
async fn only_guilds(ctx: &SlashContext<()>) -> Result<bool, DefaultError> {
    // Only pass the check if the command has been executed inside a guild
    Ok(ctx.interaction.guild_id.is_some())
}

#[check]
async fn other_check(_ctx: &SlashContext<()>) -> Result<bool, DefaultError> {
    Ok(true)
}

#[command]
#[description = "Says hello"]
// The `check` attribute accepts a comma separated list of checks, so we can add as many as we want.
// If a single one returns `false`, the command won't execute. If a check has an error and the
// command has a custom error handler, it will handle the error. Otherwise the `after` hook will
// receive the error of the check.
#[checks(only_guilds, other_check)]
async fn hello(ctx: &SlashContext<()>) -> DefaultCommandResult {
    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(String::from("Hello!")),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}
