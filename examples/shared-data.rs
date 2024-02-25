use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::task::JoinSet;
use twilight_gateway::{create_recommended, Config, EventTypeFlags, StreamExt};
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::Intents;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use twilight_model::id::Id;
use vesper::prelude::*;

pub struct Shared {
    count: AtomicUsize
}

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

    let shared = Shared {
        count: AtomicUsize::new(0)
    };

    // The builder function accepts data as the third argument, this data will then be passed to
    // every command and hook.
    let framework = Arc::new(Framework::builder(http_client, Id::new(application_id), shared)
        .group(|g| {
            g.name("count")
                .description("Shared state count related commands")
                .command(show)
                .command(increment_one)
                .command(increment_many)
        })
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

#[command]
#[description = "Shows the current count value"]
async fn show(ctx: &SlashContext<Shared>) -> DefaultCommandResult {
    let current = ctx.data.count.load(Ordering::Relaxed);

    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(format!("The count number is {current}")),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}

#[command]
#[description = "Increments the count by one"]
async fn increment_one(ctx: &SlashContext<Shared>) -> DefaultCommandResult {
    ctx.data.count.fetch_add(1, Ordering::Relaxed);

    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(format!("The count number has been incremented by one")),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}

#[command]
#[description = "Increments the count by the specified number"]
async fn increment_many(
    ctx: &SlashContext<Shared>,
    #[description = "How many numbers to add to count"] many: usize
) -> DefaultCommandResult
{
    ctx.data.count.fetch_add(many, Ordering::Relaxed);

    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(format!("The count number has been incremented by {many}")),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}
