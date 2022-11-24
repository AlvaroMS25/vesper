use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use futures_util::StreamExt;
use twilight_gateway::Cluster;
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::Intents;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use twilight_model::id::Id;
use zephyrus::prelude::*;

struct Shared {
    count: AtomicUsize
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let token = env::var("DISCORD_TOKEN").unwrap();
    let application_id = env::var("DISCORD_APPLICATION_ID").unwrap().parse::<u64>().unwrap();

    let http_client = Arc::new(Client::new(token.clone()));

    let (cluster, mut events) = Cluster::builder(token, Intents::empty())
        .http_client(Arc::clone(&http_client))
        .build()
        .await
        .unwrap();

    cluster.up().await;

    let shared = Shared {
        count: AtomicUsize::new(0)
    };

    // The builder function accepts data as the third argument, this data will then be passed to
    // every command and hook.
    let framework = Framework::builder(http_client, Id::new(application_id), shared)
        .group(|g| {
            g.name("count")
                .description("Shared state count related commands")
                .command(show)
                .command(increment_one)
                .command(increment_many)
        })
        .build();

    while let Some((_, event)) = events.next().await {
        match event {
            Event::Ready(_) => {
                framework.register_global_commands().await.unwrap();
            },
            Event::InteractionCreate(interaction) => framework.process(interaction.0).await,
            _ => ()
        }
    }
}

#[command]
#[description = "Shows the current count value"]
async fn show(ctx: &SlashContext<Shared>) -> CommandResult {
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
async fn increment_one(ctx: &SlashContext<Shared>) -> CommandResult {
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
) -> CommandResult
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
