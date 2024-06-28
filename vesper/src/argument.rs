use crate::hook::AutocompleteHook;
use crate::twilight_exports::*;
use crate::parse::Parse;
use crate::localizations::{CommandMarker, Localizations, LocalizationsProvider};
use crate::prelude::Framework;

/// A structure representing a command argument.
pub struct CommandArgument<D, T, E> {
    /// Argument name.
    pub name: &'static str,
    pub localized_names: Localizations<CommandMarker<D, T, E>>,
    /// Description of the argument.
    pub description: &'static str,
    pub localized_descriptions: Localizations<CommandMarker<D, T, E>>,
    /// Whether the argument is required.
    pub required: bool,
    /// The type this argument has.
    pub kind: CommandOptionType,
    /// The input options allowed to choose from in this command, only valid if it is [Some](Some)
    pub choices: Option<Vec<CommandOptionChoice>>,
    /// A function used to autocomplete fields.
    pub autocomplete: Option<AutocompleteHook<D>>,
    pub modify_fn: fn(&mut CommandOption)
}

impl<D, T, E> CommandArgument<D, T, E> {
    /// Converts the argument into a twilight's [command option](CommandOption)
    pub fn as_option(&self, f: &Framework<D, T, E>, c: &crate::command::Command<D, T, E>) -> CommandOption {
        let mut option = CommandOption {
            autocomplete: None,
            channel_types: None,
            choices: None,
            description: self.description.to_string(),
            description_localizations: self.localized_descriptions.get_localizations(f, c),
            kind: self.kind,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            name: self.name.to_string(),
            name_localizations: self.localized_names.get_localizations(f, c),
            options: None,
            required: Some(self.required)
        };

        (self.modify_fn)(&mut option);

        match option.kind {
            CommandOptionType::String | CommandOptionType::Integer | CommandOptionType::Number => {
                option.autocomplete = Some(self.autocomplete.is_some());
                option.choices = Some(self.choices.clone().unwrap_or_default());
            },
            _ => ()
        }

        option
    }
}

impl<D: Send + Sync, T, E> CommandArgument<D, T, E> {
    pub fn new<Arg: Parse<D>>(
        name: &'static str,
        description: &'static str,
        autocomplete: Option<AutocompleteHook<D>>
    ) -> Self
    {
        Self {
            name,
            localized_names: Default::default(),
            description,
            localized_descriptions: Default::default(),
            required: Arg::required(),
            kind: Arg::kind(),
            choices: Arg::choices(),
            autocomplete,
            modify_fn: Arg::modify_option
        }
    }

    pub fn localized_names<I, K, V>(mut self, iterator: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: ToString
    {
        self.localized_names
            .extend(iterator.into_iter().map(|(k, v)| (k.to_string(), v.to_string())));
        self
    }

    pub fn localized_names_fn(mut self, fun: CommandMarker<D, T, E>) -> Self {
        self.localized_names.set_provider(fun);
        self
    }

    pub fn localized_descriptions<I, K, V>(mut self, iterator: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: ToString
    {
        self.localized_descriptions
            .extend(iterator.into_iter().map(|(k, v)| (k.to_string(), v.to_string())));
        self
    }

    pub fn localized_descriptions_fn(mut self, fun: CommandMarker<D, T, E>) -> Self {
        self.localized_descriptions.set_provider(fun);
        self
    }
}
