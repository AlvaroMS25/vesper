use std::env;
use std::sync::Arc;
use rand::Rng;
use tokio::task::JoinSet;
use twilight_gateway::{create_recommended, Config, EventTypeFlags, StreamExt};
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::Intents;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
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
        .group(|group| {
            group.name("message")
                .description("Message related commands")
                // By setting a command here, we specify this is a command group, then multiple
                // commands can be registered using the `command` method multiple times.
                .command(repeat)
        })
        .group(|group| {
            group.name("random")
                .description("Generates random things")
                // Here, by setting another group, we are specifying this is a subcommand group.
                .group(|subgroup| {
                    subgroup.name("integer")
                        .description("Generates random integers")
                        .command(number_between)
                })
                // We can specify multiple subcommand groups.
                .group(|subgroup| {
                    subgroup.name("char")
                        .description("Generates random characters")
                        .command(random_char)
                })
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
                        let framework_clone = Arc::clone(&framework);
                        tokio::spawn(async move {
                            framework_clone.process(interaction.0).await;
                        });
                    },
                    _ => ()
                }
            }
        });
    }

    while set.join_next().await.is_some() {}
}

#[command]
#[description = "Repeats your message"]
async fn repeat(
    ctx: &SlashContext<()>,
    #[description = "Message to repeat"] message: String
) -> DefaultCommandResult
{
    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(message),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}

#[command]
#[description = "Responds with a random number between the specified range"]
async fn number_between(
    ctx: &SlashContext<()>,
    #[description = "The starting number of the range"] start: Range<i8, 0, 30>,
    #[description = "The end number of the range"] end: Range<i8, 50, 100>
) -> DefaultCommandResult
{
    // Range implements deref to the specified type, so it can be used like a normal number.
    let num = rand::thread_rng().gen_range(*start..=*end);

    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(format!("Your number is {num}")),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}

// By passing the name as an argument we are manually specifying the name of the command,
// if nothing is provided, the function name is used as the command name.
#[command("char")]
#[description = "Generates a random char"]
async fn random_char(ctx: &SlashContext<()>) -> DefaultCommandResult {
    let char = rand::random::<char>();

    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(format!("Your char is {char}")),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}
