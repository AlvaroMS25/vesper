use crate::twilight_exports::*;

/// A command argument.
pub struct CommandArgument {
    /// Argument name.
    pub name: &'static str,
    /// Description of the argument.
    pub description: &'static str,
    /// Whether the argument is required.
    pub required: bool,
    /// The type this argument has.
    pub kind: CommandOptionType,
    /// A function that allows to set specific options to the command, disabling arbitrary values.
    pub choices_fn: Box<dyn Fn() -> Option<Vec<CommandOptionChoice>> + Send>,
}

impl CommandArgument {
    pub fn as_option(&self) -> CommandOption {
        match self.kind {
            CommandOptionType::String => CommandOption::String(ChoiceCommandOptionData {
                autocomplete: false,
                choices: match (self.choices_fn)() {
                    Some(choices) => choices,
                    None => Vec::new(),
                },
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
            }),
            CommandOptionType::Integer => CommandOption::Integer(NumberCommandOptionData {
                autocomplete: false,
                choices: match (self.choices_fn)() {
                    Some(choices) => choices,
                    None => Vec::new(),
                },
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
                ..Default::default()
            }),
            CommandOptionType::Boolean => CommandOption::Boolean(BaseCommandOptionData {
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
            }),
            CommandOptionType::User => CommandOption::User(BaseCommandOptionData {
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
            }),
            CommandOptionType::Channel => CommandOption::Channel(ChannelCommandOptionData {
                channel_types: Vec::new(),
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
            }),
            CommandOptionType::Role => CommandOption::Role(BaseCommandOptionData {
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
            }),
            CommandOptionType::Mentionable => CommandOption::Mentionable(BaseCommandOptionData {
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
            }),
            CommandOptionType::Number => CommandOption::Number(NumberCommandOptionData {
                autocomplete: false,
                choices: match (self.choices_fn)() {
                    Some(choices) => choices,
                    None => Vec::new(),
                },
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
                ..Default::default()
            }),
            _ => unreachable!(),
        }
    }
}

impl
    From<(
        &'static str,
        &'static str,
        bool,
        CommandOptionType,
        Box<dyn Fn() -> Option<Vec<CommandOptionChoice>> + Send>,
    )> for CommandArgument
{
    fn from(
        (name, description, required, kind, fun): (
            &'static str,
            &'static str,
            bool,
            CommandOptionType,
            Box<dyn Fn() -> Option<Vec<CommandOptionChoice>> + Send>,
        ),
    ) -> Self {
        Self {
            name,
            description,
            required,
            kind,
            choices_fn: fun,
        }
    }
}
