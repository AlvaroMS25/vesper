use std::env;
use std::sync::Arc;
use tokio::task::JoinSet;
use twilight_gateway::{create_recommended, Config, EventTypeFlags, StreamExt};
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::Intents;
use twilight_model::id::Id;
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
        .command(show_modal)
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
#[description = "Shows a modal"]
async fn show_modal(ctx: &SlashContext<()>) -> DefaultCommandResult {
    let wait = ctx.create_modal::<MyModal>().await?;
    let output = wait.await?;
    println!("{output:?}");
    Ok(())
}

#[derive(Modal, Debug)]
struct MyModal {
    #[modal(placeholder = "My placeholder")]
    something: String,
    #[modal(label = "Another field", paragraph)]
    field_2: Option<String>
}
