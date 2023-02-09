use std::env;
use std::sync::Arc;
use futures_util::StreamExt;
use twilight_gateway::{stream::{self, ShardEventStream}, Config};
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::Intents;
use twilight_model::id::Id;
use zephyrus::prelude::*;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").unwrap();
    let application_id = env::var("DISCORD_APPLICATION_ID").unwrap().parse::<u64>().unwrap();

    let http_client = Arc::new(Client::new(token.clone()));

    let config = Config::new(token.clone(), Intents::empty());
    let mut shards = stream::create_recommended(
        &http_client,
        config,
        |_, builder| builder.build()
    ).await.unwrap().collect::<Vec<_>>();

    let mut stream = ShardEventStream::new(shards.iter_mut());

    let framework = Framework::builder(http_client, Id::new(application_id), ())
        .command(show_modal)
        .build();

    while let Some((_, event)) = stream.next().await {
        match event {
            Err(error) => {
                if error.is_fatal() {
                    eprintln!("Gateway connection fatally closed, error: {error:?}");
                    break;
                }
            },
            Ok(event) => match event {
                Event::Ready(_) => {
                    // We have to register the commands for them to show in discord.
                    framework.register_global_commands().await.unwrap();
                },
                Event::InteractionCreate(interaction) => framework.process(interaction.0).await,
                _ => ()
            }
        }
    }
}

#[command]
#[description = "Shows a modal"]
async fn show_modal(ctx: &SlashContext<()>) -> DefaultCommandResult {
    let wait = ctx.create_modal::<MyModal>().await?;
    let output = wait.await?;
    println!("{output:?}");
    Ok(())
}

#[derive(Modal, Debug)]
struct MyModal {
    #[placeholder = "My placeholder"]
    something: String,
    #[label = "Another field"]
    #[paragraph]
    field_2: Option<String>
}
