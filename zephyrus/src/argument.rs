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

/// A command argument.
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
    pub fn as_option(&self) -> CommandOption {
        match self.kind {
            CommandOptionType::String => CommandOption::String(ChoiceCommandOptionData {
                autocomplete: self.autocomplete.is_some(),
                choices: self.choices.clone().unwrap_or_default(),
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
                ..Default::default()
            }),
            CommandOptionType::Integer => CommandOption::Integer(NumberCommandOptionData {
                autocomplete: self.autocomplete.is_some(),
                choices: self.choices.clone().unwrap_or_default(),
                description: self.description.to_string(),
                max_value: self.limits.unwrap_or_default().max,
                min_value: self.limits.unwrap_or_default().min,
                name: self.name.to_string(),
                required: self.required,
                ..Default::default()
            }),
            CommandOptionType::Boolean => CommandOption::Boolean(BaseCommandOptionData {
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
                ..Default::default()
            }),
            CommandOptionType::User => CommandOption::User(BaseCommandOptionData {
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
                ..Default::default()
            }),
            CommandOptionType::Channel => CommandOption::Channel(ChannelCommandOptionData {
                channel_types: Vec::new(),
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
                ..Default::default()
            }),
            CommandOptionType::Role => CommandOption::Role(BaseCommandOptionData {
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
                ..Default::default()
            }),
            CommandOptionType::Mentionable => CommandOption::Mentionable(BaseCommandOptionData {
                description: self.description.to_string(),
                name: self.name.to_string(),
                required: self.required,
                ..Default::default()
            }),
            CommandOptionType::Number => CommandOption::Number(NumberCommandOptionData {
                autocomplete: self.autocomplete.is_some(),
                choices: self.choices.clone().unwrap_or_default(),
                description: self.description.to_string(),
                max_value: self.limits.unwrap_or_default().max,
                min_value: self.limits.unwrap_or_default().min,
                name: self.name.to_string(),
                required: self.required,
                ..Default::default()
            }),
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
