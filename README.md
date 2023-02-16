# Zephyrus Framework [![Crate](https://img.shields.io/crates/v/zephyrus)](https://crates.io/crates/zephyrus)

`Zephyrus` is a slash command framework meant to be used by [twilight](https://twilight.rs/)

**Note**: The framework is new and might have some problems, all contributions are appreciated

<strong>This crate is independent from the [twilight](https://twilight.rs/) ecosystem</strong>


***The framework is experimental and the API might change.***
***

`Zephyrus` is a command framework which uses slash commands, it mainly offers variable argument parsing.

Parsing is done with the `Parse` trait, so users can implement the parsing of their own types.

Argument parsing is done in a named way, this means the argument name shown on discord gets parsed into
the arguments named the same way in the handler function.

The framework itself ***doesn't*** spawn any tasks by itself, so you might want to wrap it in an `Arc` and call
`tokio::spawn` before calling the `.process` method.
***

## Usage example

```rust
use std::sync::Arc;
use futures::StreamExt;
use twilight_gateway::{Cluster, cluster::Events};
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::Intents;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use twilight_model::id::Id;
use twilight_model::id::marker::{ApplicationMarker, GuildMarker};
use zephyrus::prelude::*;

#[command]
#[description = "Says hello"]
async fn hello(ctx: &SlashContext<()>) -> DefaultCommandResult {
    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(String::from("Hello world")),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}

async fn handle_events(http_client: Arc<Client>, mut events: Events, app_id: Id<ApplicationMarker>) {
    let framework = Arc::new(Framework::builder(http_client, app_id, ())
        .command(hello)
        .build());

    // Zephyrus can register commands in guilds or globally.
    framework.register_guild_commands(Id::<GuildMarker>::new("<GUILD_ID>")).await.unwrap();

    while let Some((_, event)) = events.next().await {
        match event {
            Event::InteractionCreate(i) => {
                let clone = Arc::clone(&framework);
                tokio::spawn(async move {
                    let inner = i.0;
                    clone.process(inner).await;
                });
            },
            _ => (),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let token = std::env::var("DISCORD_TOKEN")?;
    let app_id = Id::<ApplicationMarker>::new(std::env::var("APP_ID")?.parse()?);
    let client = Arc::new(Client::builder().token(token.clone()).build());

    let (cluster, events) = Cluster::builder(token, Intents::empty())
        .http_client(Arc::clone(&client))
        .build()
        .await?;

    cluster.up().await;
    handle_events(client, events, app_id).await;

    Ok(())
}
```

# Usage guide

***

# Creating commands

Every command is an ``async`` function, having always as the first parameter a `&SlashContext<T>`

```rust
#[command]
#[description = "This is the description of the command"]
async fn command(
    ctx: &SlashContext</* Your type of context*/>, // The context must always be the first parameter.
    #[description = "A description for the argument"] some_arg: String,
    #[rename = "other_arg"] #[description = "other description"] other: Option<UserId>
) -> DefaultCommandResult 
{
    // Command body
    
    Ok(())
}
```

### Command functions

Command functions must include a `description` attribute, which will be seen in discord when the user tries to use the command.

The `#[command]` attribute also allows to rename the command by passing the name of the command to the attribute like
`#[command("Command name here")]`. If the name is not provided, the command will use the function name.

### Command arguments

Command arguments are very similar to command functions, they also need a `#[description]` attribute that will be seen
in discord by the user when filling up the command argument.

As shown in the example, a `#[rename]` attribute can also be used, this will change the name of the argument seen in 
discord. If the attribute is not used, the argument will have the same name as in the function.

**Important: All command functions must have as the first parameter a `&SlashContext<T>`**

## Setting choices as command arguments
Choices are a very useful feature of slash commands, allowing the developer to set some choices from which the user has
to choose.

Zephyrus allows doing this in an easy way, to allow this, a derive macro is provided by the framework. This macro is
named the same way as `Parse` trait and can only be used in enums to define the options. Renaming is also allowed here
by using the `#[parse(rename)]` attribute and allows to change the option name seen in discord.

```rust
#[derive(Parse)]
enum Choices {
    First,
    Second,
    Third,
    #[parse(rename = "Forth")]
    Other
}

#[command]
#[description = "Some description"]
async fn choices(
    ctx: &SlashContext<()>,
    #[description = "Some description"] choice: Choices
) -> DefaultCommandResult
{
    // Command body
    Ok(())
}
```

## Autocompleting commands
Autocomplete user input is made easy with `Zephyrus`, just use the `autocomplete` macro provided by the framework.

Here, take a look at this example. We'll use as the base an empty command like this

```rust
#[command]
#[description = "Some description"]
async fn some_command(
    ctx: &SlashCommand</* Some type */>,
    #[autocomplete = "autocomplete_arg"] #[description = "Some description"] arg: String
) -> DefaultCommandResult
{
    // Logic goes here
    Ok(())
}
```

As you may have noticed, we added an `autocomplete` attribute to the argument `arg`. The input specified on it must
point to a function marked with the `#[autocomplete]` attribute like this one:

```rust
#[autocomplete]
async fn autocomplete_arg(ctx: AutocompleteContext</* Some type */>) -> Option<InteractionResponseData> {
    // Function body
}
```

Autocompleting functions must have an `AutocompleteContext<T>` as the sole parameter, it allows you to access to the
data stored at the framework while also allowing you to access the raw interaction, the framework's http client and the
user input, if exists.

## Permissions
To specify required permissions to run a command, just use the `#[required_permissions]` attribute when declaring
a command, or the `.required_permissions` method when declaring a command group.

The attribute accepts as input a comma separated list of 
[twilight's permissions](https://docs.rs/twilight-model/latest/twilight_model/guild/struct.Permissions.html). Let's take
a look at what it would look like to create a command needing `MANAGE_CHANNELS` and `MANAGE_MESSAGES` permissions:

```rust
#[command]
#[description = "Super cool command"]
#[required_permissions(MANAGE_CHANNELS, MANAGE_MESSAGES)]
async fn super_cool_command(ctx: &SlashContext</* Your type */>) -> DefaultCommandResult {
    // Body
    Ok(())
}
```

***

# Command Groups

`Zephyrus` supports both `SubCommands` and `SubCommandGroups` by default.

To give examples, let's say we have created the following command:

```rust
#[command]
#[description = "Something"]
async fn something(ctx: &SlashContext</* Your type */>) -> DefaultCommandResult {
    // Command block
    Ok(())
}
```

With this we can now create both subcommands and subcommand groups

## Creating subcommands

To create a subcommand you need to create a group, then you can add all the subcommands.

```rust
#[tokio::main]
async fn main() {
    let framework = Framework::builder()
        .group(|g| {
            g.name("<GROUP_NAME>")
                .description("<GROUP_DESCRIPTION>")
                .add_command(something)
                .add_command(..)
                ..
        })
        .build();
}
```

## Creating subcommand groups

Subcommand groups are very similar to subcommands, they are created almost the same way, but instead of using
`.add_command` directly, we have to use `.group` before to register a group.

```rust
#[tokio::main]
async fn main() {
    let framework = Framework::builder()
        .group(|g| {
            g.name("<GROUP_NAME>")
                .description("<GROUP_DESCRIPTION>")
                .group(|sub| { // With this we have created a subcommand group.
                    sub.name("<SUBGROUP_NAME>")
                        .description("<SUBGROUP_DESCRIPTION>")
                        .add_command(something)
                        .add_command(..)
                        ..
                })
        })
        .build();
}
```

***

# Hooks
There are three hooks available, `before`, `after` and `error_handler`.

## Before

The before hook is triggered before the command and has to return a `bool` indicating if the command should be executed or not.

```rust
#[before]
async fn before_hook(ctx: &SlashContext</*Your type*/>, command_name: &str) -> bool {
    // Do something
    
    true // <- if we return true, the command will be executed normally.
}
```


## After

The after hook is triggered after the command execution, and it provides the result of the command.

```rust
#[after]
async fn after_hook(ctx: &SlashContext</* Your type */>, command_name: &str, result: Option<DefaultCommandResult>) {
    // Do something with the result.
}
```

## Specific error handling

Commands can have specific error handlers. When an error handler is set to a command, if the command (or any of its checks)
fails, the error handler will be called, and the `after` hook will receive `None` as the third argument. However, in case
the command execution finishes without raising errors, the `after` hook will receive the result of the command.

Let's take a look at a simple implementation:

```rust
#[error_handler]
async fn handle_ban_error(_ctx: &SlashContext</* Some type */>, error: DefaultError) {
    println!("The ban command had an error");
    
    // Handle the error
}


#[command]
#[description = "Tries to ban the bot itself, raising an error"]
#[error_handler(handle_ban_error)]
async fn ban_itself(ctx: &SlashContext</* Some type */>) -> DefaultCommandResult {
    // A bot cannot ban itself, so this will result in an error.
    ctx.http_client().ban(ctx.interaction.guild_id.unwrap(), Id::new(ctx.application_id.get()))
        .await?;
    
    Ok(())
}
```

Since the command will always fail because a bot cannot ban itself, the error handler will be called everytime the command
executes, thus passing `None` to the `after` hook if set.

***

# Checks

Checks are pretty similar to the ``Before`` hook, but unlike it, they are not global. Instead, they need to be assigned
to each command.

Let's take a look on how to use it:

Let's create some checks like this:

```rust
#[check]
async fn only_guilds(ctx: &SlashContext</* Some type */>) -> Result<bool, DefaultError> {
    // Only execute the command if we are inside a guild.
    Ok(ctx.interaction.guild_id.is_some())
}

#[check]
async fn other_check(_ctx: &SlashContext</* Some type */>) -> Result<bool, DefaultError> {
    // Some other check here.
    Ok(true)
}
```

Then we can assign them to our command using the ``check`` attribute, which accepts a comma separated list of checks:
```rust
#[command]
#[description = "Some description"]
#[checks(only_guilds, other_check)]
async fn my_command(ctx: &SlashContext</* Some type */>) -> DefaultCommandResult {
    // Do something
    Ok(())
}
```

***

# Using custom return types

The framework allows the user to specify what types to return from command/checks execution. The framework definition is
like this:

```rust
pub struct Framework<D, T = (), E = DefaultError>
```

Where `D` is the type of the data held by the framework and `T` and `E` are the return types of a command in form of
`Result<T, E>`, however, specifying custom types is optional, and the framework provides a `DefaultCommandResult` and
`DefaultError` for those who don't want to have a custom error.

The types of the `after`, `error_handler` and `check` hook arguments change accordingly to the generics specified in 
the framework, so their signatures could be interpreted like this:

After hook:
```rust
async fn(&SlashContext</* Some type */>, &str, Option<Result<T, E>>)
```

Error handler hook:
```rust
async fn(&SlashContext</* Some type */>, E)
```

Command checks:
```rust
async fn(&SlashContext</* Some type */>) -> Result<bool, E>
```

Note that those are not the real signatures, since the functions return `Box`ed futures.
