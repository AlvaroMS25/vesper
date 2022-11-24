use std::env;
use std::sync::Arc;
use futures_util::StreamExt;
use twilight_gateway::Cluster;
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::Intents;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use twilight_model::id::Id;
use zephyrus::prelude::*;

#[tokio::main]
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

    // Wrap the framework in an Arc, so we can share it across tasks.
    let framework = Arc::new(Framework::builder(http_client, Id::new(application_id), ())
        .command(state)
        .build());

    while let Some((_, event)) = events.next().await {
        match event {
            Event::Ready(_) => {
                framework.register_global_commands().await.unwrap();
            },
            Event::InteractionCreate(interaction) => {
                let framework_clone = Arc::clone(&framework);

                // The framework doesn't spawn any tasks by itself, so if we want it to run in a
                // separate task, we have to spawn it manually.
                tokio::spawn(async move {
                    framework_clone.process(interaction.0).await;
                });
            },
            _ => ()
        }
    }
}

#[command]
#[description = "Says where the client is running"]
async fn state(ctx: &SlashContext<()>) -> CommandResult {
    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(String::from("Running inside a multithreaded tokio runtime!")),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}
