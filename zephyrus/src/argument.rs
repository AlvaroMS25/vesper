use crate::hook::AutocompleteHook;
use crate::twilight_exports::*;
use crate::parse::Parse;
use std::collections::HashMap;

/// A structure representing a command argument.
pub struct CommandArgument<D> {
    /// Argument name.
    pub name: &'static str,
    pub localized_names: Option<HashMap<String, String>>,
    /// Description of the argument.
    pub description: &'static str,
    pub localized_descriptions: Option<HashMap<String, String>>,
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

impl<D> CommandArgument<D> {
    /// Converts the argument into a twilight's [command option](CommandOption)
    pub fn as_option(&self) -> CommandOption {
        let mut option = CommandOption {
            autocomplete: None,
            channel_types: None,
            choices: None,
            description: self.description.to_string(),
            description_localizations: None,
            kind: self.kind,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            name: self.name.to_string(),
            name_localizations: None,
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

impl<D: Send + Sync> CommandArgument<D> {
    pub fn new<T: Parse<D>>(
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
            required: T::required(),
            kind: T::kind(),
            choices: T::choices(),
            autocomplete,
            modify_fn: T::modify_option
        }
    }

    pub fn localized_names<I, L>(mut self, iterator: I) -> Self 
    where
        I: IntoIterator<Item = (L, L)>,
        L: ToString
    {
        if self.localized_names.is_none() {
            self.localized_names = Some(Default::default());
        }

        self.localized_names.as_mut()
            .unwrap()
            .extend(iterator.into_iter().map(|(k, v)| (k.to_string(), v.to_string())));
        self
    }

    pub fn localized_descriptions<I, L>(mut self, iterator: I) -> Self 
    where
        I: IntoIterator<Item = (L, L)>,
        L: ToString
    {
        if self.localized_descriptions.is_none() {
            self.localized_descriptions = Some(Default::default());
        }

        self.localized_descriptions.as_mut()
            .unwrap()
            .extend(iterator.into_iter().map(|(k, v)| (k.to_string(), v.to_string())));
        self
    }
}
