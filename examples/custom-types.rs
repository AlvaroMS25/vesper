use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use futures_util::StreamExt;
use twilight_gateway::{stream::{self, ShardEventStream}, Config};
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::Intents;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use twilight_model::id::Id;
use zephyrus::framework::DefaultError;
use zephyrus::prelude::*;

// The framework accepts custom error types, however, the custom error must implement
// `From<ParseError>`
pub enum MyError {
    Parse(ParseError),
    Http(twilight_http::Error),
    Other(DefaultError)
}

impl From<ParseError> for MyError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl From<twilight_http::Error> for MyError {
    fn from(value: twilight_http::Error) -> Self {
        Self::Http(value)
    }
}

impl From<DefaultError> for MyError {
    fn from(value: DefaultError) -> Self {
        Self::Other(value)
    }
}

// Let's measure the time a command needs to respond to an interaction, to do this let's specify
// a custom `Ok` return type for our commands.
pub struct ElapsedTime(Duration);

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

    // The framework supports setting a custom return type for commands and some hooks, to specify
    // them just pass them as generic types.
    let framework = Framework::<_, ElapsedTime, MyError>::builder(http_client, Id::new(application_id), ())
        .command(timer)
        .after(after_hook)
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

#[after]
async fn after_hook(
    _: &SlashContext<()>,
    command_name: &str,
    result: Option<Result<ElapsedTime, MyError>>
) {
    // We don't have a custom error handler, so result will be always `Some`
    let result = result.unwrap();

    match result {
        Ok(elapsed) => {
            println!("Command {} took {} ms to execute", command_name, elapsed.0.as_millis())
        },
        Err(e) => match e {
            MyError::Parse(p) => println!("An error occurred when parsing a command {}", p),
            MyError::Http(e) => println!("An HTTP error occurred: {}", e),
            MyError::Other(other) => println!("An error occurred {}", other)
        }
    };
}

#[command]
#[description = "Measures the time needed to respond to an interaction"]
async fn timer(ctx: &SlashContext<()>) -> Result<ElapsedTime, MyError> {
    let start = Instant::now();

    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(String::from("")),
                ..Default::default()
            })
        }
    ).await?;

    Ok(ElapsedTime(Instant::now() - start))
}
