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

    let framework = Framework::builder(http_client, Id::new(application_id), ())
        .command(hello)
        .before(before_hook)
        .after(after_hook)
        .build();

    while let Some((_, event)) = events.next().await {
        match event {
            Event::Ready(_) => {
                // We have to register the commands for them to show in discord.
                framework.register_global_commands().await.unwrap();
            },
            Event::InteractionCreate(interaction) => framework.process(interaction.0).await,
            _ => ()
        }
    }
}

#[before]
async fn before_hook(_ctx: &SlashContext<()>, command_name: &str) -> bool {
    println!("Before hook executed for command {command_name}");
    // The return type of this function specifies if the actual command should run or not, if `false`
    // is returned, then the command won't execute.
    true
}

// The result field will be some only if the command returned no errors or if the command has
// no custom error handler set.
#[after]
async fn after_hook(_ctx: &SlashContext<()>, command_name: &str, result: Option<CommandResult>) {
    println!("{command_name} finished, returned value: {result:?}");
}

#[command]
#[description = "Says hello"]
async fn hello(ctx: &SlashContext<()>) -> CommandResult {
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

#[error_handler]
async fn handle_error(_ctx: &SlashContext<()>, result: CommandResult) {
    println!("Command had an error: {result:?}");
}

#[command]
#[description = "Tries to ban the bot itself and raises an error"]
// As we registered here a custom error handler, the after hook will only only have `Some` in the
// result argument if the command execution finishes without raising any exceptions, which in this
// case is only if the command is executed outside of a guild. Otherwise the result argument
// will be `None`, as the error will be consumed at the custom error handler.
#[error_handler(handle_error)]
async fn raises_error(ctx: &SlashContext<()>) -> CommandResult {
    ctx.acknowledge().await?;
    if !ctx.interaction.is_guild() {
        ctx.update_response(|res| {
            res.content(Some("This command can only be used in guilds")).unwrap()
        }).await?;

        return Ok(())
    }

    // Trying to ban a bot by itself results in an error.
    ctx.http_client().ban(ctx.interaction.guild_id.unwrap(), Id::new(ctx.application_id.get()))
        .await?;

    Ok(())
}
