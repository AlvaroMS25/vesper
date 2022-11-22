use crate::hook::AutocompleteHook;
use crate::twilight_exports::*;
use twilight_model::application::command::CommandOptionValue;
use crate::parse::Parse;

/// The constraints the arguments impose to the user.
/// This is normally provided by implementing [parse](crate::parse::Parse) into a type.
#[derive(Copy, Clone, Default)]
pub struct ArgumentLimits {
    pub min: Option<CommandOptionValue>,
    pub max: Option<CommandOptionValue>
}

/// A structure representing a command argument.
pub struct CommandArgument<D> {
    /// Argument name.
    pub name: &'static str,
    /// Description of the argument.
    pub description: &'static str,
    /// Whether the argument is required.
    pub required: bool,
    /// The type this argument has.
    pub kind: CommandOptionType,
    /// The input options allowed to choose from in this command, only valid if it is [Some](Some)
    pub choices: Option<Vec<CommandOptionChoice>>,
    /// The input limits of this argument.
    pub limits: Option<ArgumentLimits>,
    /// A function used to autocomplete fields.
    pub autocomplete: Option<AutocompleteHook<D>>,
}

impl<D> CommandArgument<D> {
    /// Converts the argument into a twilight's [command option](CommandOption)
    pub fn as_option(&self) -> CommandOption {
        match self.kind {
            CommandOptionType::String => CommandOption {
                kind: CommandOptionType::String,
                autocomplete: Some(self.autocomplete.is_some()),
                choices: Some(self.choices.clone().unwrap_or_default()),
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: Some(self.required),
                channel_types: None,
                description_localizations: None,
                max_length: None,
                max_value: None,
                min_length: None,
                min_value: None,
                name_localizations: None,
                options: None,
            },
            CommandOptionType::Integer => CommandOption {
                kind: CommandOptionType::Integer,
                autocomplete: Some(self.autocomplete.is_some()),
                choices: Some(self.choices.clone().unwrap_or_default()),
                description: self.description.to_string(),
                max_value: self.limits.unwrap_or_default().max,
                min_value: self.limits.unwrap_or_default().min,
                name: self.name.to_string(),
                required: Some(self.required),
                channel_types: None,
                description_localizations: None,
                max_length: None,
                min_length: None,
                name_localizations: None,
                options: None,
            },
            CommandOptionType::Boolean => CommandOption {
                kind: CommandOptionType::Boolean,
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: Some(self.required),
                autocomplete: None,
                choices: None,
                channel_types: None,
                description_localizations: None,
                max_length: None,
                max_value: None,
                min_length: None,
                min_value: None,
                name_localizations: None,
                options: None,
            },
            CommandOptionType::User => CommandOption {
                kind: CommandOptionType::User,
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: Some(self.required),
                autocomplete: None,
                choices: None,
                channel_types: None,
                description_localizations: None,
                max_length: None,
                max_value: None,
                min_length: None,
                min_value: None,
                name_localizations: None,
                options: None,
            },
            CommandOptionType::Channel => CommandOption {
                kind: CommandOptionType::Channel,
                channel_types: Some(Vec::new()),
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: Some(self.required),
                autocomplete: None,
                choices: None,
                description_localizations: None,
                max_length: None,
                max_value: None,
                min_length: None,
                min_value: None,
                name_localizations: None,
                options: None,
            },
            CommandOptionType::Role => CommandOption {
                kind: CommandOptionType::Role,
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: Some(self.required),
                autocomplete: None,
                choices: None,
                channel_types: None,
                description_localizations: None,
                max_length: None,
                max_value: None,
                min_length: None,
                min_value: None,
                name_localizations: None,
                options: None,
            },
            CommandOptionType::Mentionable => CommandOption {
                kind: CommandOptionType::Mentionable,
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: Some(self.required),
                autocomplete: None,
                choices: None,
                channel_types: None,
                description_localizations: None,
                max_length: None,
                max_value: None,
                min_length: None,
                min_value: None,
                name_localizations: None,
                options: None,
            },
            CommandOptionType::Number => CommandOption {
                kind: CommandOptionType::Number,
                autocomplete: Some(self.autocomplete.is_some()),
                choices: Some(self.choices.clone().unwrap_or_default()),
                description: self.description.to_string(),
                max_value: self.limits.unwrap_or_default().max,
                min_value: self.limits.unwrap_or_default().min,
                name: self.name.to_string(),
                required: Some(self.required),
                channel_types: None,
                description_localizations: None,
                max_length: None,
                min_length: None,
                name_localizations: None,
                options: None,
            },
            _ => unreachable!(),
        }
    }
}

impl<D: Send + Sync> CommandArgument<D> {
    pub fn new<T: Parse<D>>(
        name: &'static str,
        description: &'static str,
        autocomplete: Option<AutocompleteHook<D>>
    ) -> Self
    {
        Self {
            name,
            description,
            required: T::required(),
            kind: T::kind(),
            choices: T::choices(),
            limits: T::limits(),
            autocomplete
        }
    }
}
