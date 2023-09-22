# 🎉🎉 We're changing names! 0.11.0 Will be the last release under the name `zephyrus`, next releases (including 0.11.0) will be available under the name `vesper` in crates.io 🎉🎉

# Zephyrus Framework [![Crate](https://img.shields.io/crates/v/zephyrus)](https://crates.io/crates/zephyrus)

`Zephyrus` is a slash command framework meant to be used by [twilight](https://twilight.rs/)

**Note**: The framework is new and might have some problems, all contributions are appreciated

<strong>This crate is independent from the [twilight](https://twilight.rs/) ecosystem</strong>

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
use futures_util::StreamExt;
use twilight_gateway::{stream::{self, ShardEventStream}, Config};
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

async fn handle_events(http_client: Arc<Client>, mut events: ShardEventStream, app_id: Id<ApplicationMarker>) {
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
    let client = Arc::new(Client::new(token.clone()));

    let config = Config::new(token, Intents::empty());
    let mut shards = stream::create_recommended(
        &client,
        config,
        |_, builder| builder.build()
    ).await.unwrap().collect::<Vec<_>>();
    let mut shard_stream = ShardEventStream::new(shards.iter_mut());

    handle_events(client, shard_stream, app_id).await;

    Ok(())
}
```

# Usage guide

***

# Creating commands

Every command is an ``async`` function, having always as the first parameter a `&SlashContext<T>`

The framework supports `chat`, `message` and `user` commands, let's take a look at each of them

#### Chat command
```rust
#[command(chat)] // or #[command]
#[description = "This is the description of the command"]
async fn command(
    ctx: &SlashContext</* Your type of context*/>, // The context must always be the first parameter.
    #[description = "A description for the argument"] some_arg: String,
    #[rename = "other_arg"] #[description = "other description"] other: Option<Id<UserMarker>>
) -> DefaultCommandResult 
{
    // Command body
    
    Ok(())
}
```

#### User command
```rust
#[command(user)]
#[description = "This is the description of the command"]
async fn command(
    ctx: &SlashContext</* Your type of context*/>, // The context must always be the first parameter.
) -> DefaultCommandResult 
{
    // Command body
    
    Ok(())
}
```

#### Message command
```rust
#[command(message)]
#[description = "This is the description of the command"]
async fn command(
    ctx: &SlashContext</* Your type of context*/>, // The context must always be the first parameter.
) -> DefaultCommandResult 
{
    // Command body
    
    Ok(())
}
```

As you can see, the only difference between them is the usage of `#[command({chat, user, message})` and the fact that only
`chat` commands can take arguments.

The `command` macro defaults to a `chat` command, so if none of `{chat, user, message}` specifiers is used, the macro
will treat it as a `chat` command, so `#[command]` is equivalent to `#[command(chat)]`.

**If a non-chat command takes arguments in it's handler, the framework will allow it, but it won't send them to discord.**

The framework also provides a `#[only_guilds]` attribute which will mark the command to only be available on guilds and
an `#[nsfw]` for nsfw commands.

The same command used before as an example could be marked only for guilds/nsfw the following way:
```rust
#[command]
#[nsfw] // This command is now marked as nsfw
#[description = "This is the description of the command"]
async fn command(
    ctx: &SlashContext</* Your type of context*/>,
    #[description = "A description for the argument"] some_arg: String,
    #[rename = "other_arg"] #[description = "other description"] other: Option<Id<UserMarker>>
) -> DefaultCommandResult 
{
    // Command body
    
    Ok(())
}

#[command(chat)]
#[only_guilds] // This command is now only marked as only available inside of guilds
#[description = "This is the description of the command"]
async fn command(
    ctx: &SlashContext</* Your type of context*/>,
    #[description = "A description for the argument"] some_arg: String,
    #[rename = "other_arg"] #[description = "other description"] other: Option<Id<UserMarker>>
) -> DefaultCommandResult
{
    // Command body

    Ok(())
}

#[command(chat)]
#[only_guilds] // This command is now marked as nsfw and only available inside guilds
#[nsfw]
#[description = "This is the description of the command"]
async fn command(
    ctx: &SlashContext</* Your type of context*/>, // The context must always be the first parameter.
    #[description = "A description for the argument"] some_arg: String,
    #[rename = "other_arg"] #[description = "other description"] other: Option<Id<UserMarker>>
) -> DefaultCommandResult
{
    // Command body

    Ok(())
}
```

### Using localizations
The framework allows localizations in commands and its arguments, to do this we have `#[localized_names]` and `#[localized_descriptions]`
attributes, these attributes accept a comma separated list of items. Let's take a look at them:

***Locales must be valid, to see them, refer to [Discord locales reference](https://discord.com/developers/docs/reference#locales)***
```rust
#[command]
#[localized_names("en-US" = "US name", "en-GB" = "GB name", "es-ES" = "Spanish name")]
#[localized_descriptions("en-US" = "US description", "en-GB" = "GB description", "es-ES" = "Spanish description")]
#[description = "My description"]
async fn my_localized_command(
    ctx: &SlashContext</* Data type */>,
    #[localized_names("en-US" = "US name", "en-GB" = "GB name", "es-ES" = "Spanish name")]
    #[description = "Another description"]
    #[localized_descriptions("en-US" = "US description", "en-GB" = "GB description", "es-ES" = "Spanish description")]
    my_argument: String
) -> DefaultCommandResult
{
    // Code here
    Ok(())
}
```


### Command functions

Command functions must include a `description` attribute, which will be seen in discord when the user tries to use the command.

The `#[command]` macro also allows to rename the command by passing the name of the command to the attribute like
`#[command({chat, user, message}, name = "Command name here")]`. If the name is not provided, the command will use the 
function name.

If using the short form of `#[command]` while creating a `chat` command, the rename can be passed directly like
`#[command("Command name")]`, that is equivalent to `#[command(chat, name = "Command name")]`

### Command arguments

Command arguments are very similar to command functions, they also need a `#[description]` attribute that will be seen
in discord by the user when filling up the command argument.

As shown in the example, a `#[rename]` attribute can also be used, this will change the name of the argument seen in 
discord. If the attribute is not used, the argument will have the same name as in the function.

Arguments can also be marked with a `#[skip]` attribute. Arguments marked as `#[skip]` don't allow`#[description]`
nor`#[rename]` attributes and won't be seen in discord when using the command, but they will be parsed by the framework. This can
be useful for extracting data that has nothing to do with the command input from the interaction. Let's take a look
at an example:

```rust
pub struct ExtractSomething {
    //...
}

#[async_trait]
impl Parse<T> for ExtractSomething
where T: Send + Sync
{
    async fn parse(
        http_client: &WrappedClient,
        data: &T,
        value: Option<&CommandOptionValue>, // <- will be empty since the option was not sent
        resolved: Option<&mut CommandInteractionDataResolved>
    ) -> Result<Self, ParseError>
    {
        // implement parsing logic
    }

    fn kind() -> CommandOptionType {
        // Since the struct will be marked as #[skip] this method won't be used.
        unreachable!()
    }
}

#[command]
#[description = "Something here"]
async fn my_command(
    ctx: &SlashContext</* Data type */>,
    #[skip] my_extractor: ExtractSomething // This won't be seen on discord, but will be parsed
) -> DefaultCommandResult
{
    // Command logic here
    Ok(())
}

```

### **Important: All command functions must have as the first parameter a `&SlashContext<T>`**

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

# Modals

Since version 0.8.0, the framework provides a derive macro to make modals as easy as possible. Let's take a look at
an example:

```rust
use zephyrus::prelude::*;

#[derive(Modal, Debug)]
#[modal(title = "Test modal")]
struct MyModal {
    field: String,
    #[modal(paragraph, label = "Paragraph")]
    paragraph: String,
    #[modal(placeholder = "This is an optional field")]
    optional: Option<String>
}

#[command]
#[description = "My command description"]
async fn my_command(ctx: &SlashContext</* Some type */>) -> DefaultCommandResult {
    let modal_waiter = ctx.create_modal::<MyModal>().await?;
    let output = modal_waiter.await?;
    
    println!("{output:?}");
    
    Ok(())
}

```

Here the ´Modal´ derive macro derives the modal trait which allows us to create them, then we can modify how it will be
shown to the user using the `#[modal(..)}` attributes. To see the full list of allowed attributes, take a look at the
[macro declaration].

Currently, only `String` and `Option<String>` fields are allowed.

[macro declaration]: https://github.com/AlvaroMS25/zephyrus/blob/master/zephyrus-macros/src/lib.rs#L150-L236

# Bulk Commands Overwrite
If you'd like to use Discord's [Bulk Overwrite Global Application Commands](https://discord.com/developers/docs/interactions/application-commands#bulk-overwrite-global-application-commands) enpoint, perhaps in tandem with a [commands lockfile](https://github.com/carterhimmel/thoth/tree/28c3855b1c55c9ed839bbbcbf9e9c704bf2bd81a/.github/workflows/cd_commands.yml), you'll want to use `Framework#twilight_commands`.

> **Note**
> This requires the `bulk` feature.

```rust
fn create_framework(
    http_client: Arc<Client>,
    app_id: Id<ApplicationMarker>
) -> Framework<()> {
    Framework::builder(http_client, app_id, ())
        .command(hello)
        .build()
}

fn create_lockfile(framework: Framework<()>) -> Result<()> {
    let commands = framework.twilight_commands();
    let content = serde_json::to_string_pretty(&commands)?;

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/commands.lock.json").to_string();
	std::fs::write(path, content).unwrap();

    Ok(())
}
```
