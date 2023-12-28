use crate::builder::WrappedClient;
use crate::context::SlashContext;
use crate::parse::{Parse, ParseError};
use crate::twilight_exports::{InteractionData, CommandDataOption, CommandOptionType, CommandOptionValue, CommandInteractionDataResolved};

/// An iterator used to iterate through slash command options.
pub struct DataIterator<'a, D> {
    src: Vec<&'a CommandDataOption>,
    resolved: &'a mut Option<CommandInteractionDataResolved>,
    http: &'a WrappedClient,
    data: &'a D
}

impl<'a, D> DataIterator<'a, D> {
    /// Creates a new [iterator](self::DataIterator) at the given source.
    pub fn new(ctx: &'a mut SlashContext<'_, D>) -> Self {
        let data = match ctx.interaction.data.as_mut().unwrap() {
            InteractionData::ApplicationCommand(data) => data,
            _ => unreachable!()
        };

        Self {
            src: Self::get_data(&data.options),
            resolved: &mut data.resolved,
            http: ctx.http_client,
            data: ctx.data
        }
    }
}

impl<'a, D: 'a> DataIterator<'a, D> {
    /// Gets the first value which satisfies the given predicate.
    pub fn get<F>(&mut self, predicate: F) -> Option<&'a CommandDataOption>
    where
        F: Fn(&CommandDataOption) -> bool,
    {
        let i = {
            let mut idx = 0;
            let mut found = false;

            while idx < self.src.len() && !found {
                if predicate(self.src[idx]) {
                    found = true;
                }

                if !found {
                    idx += 1;
                }
            }

            if found {
                Some(idx)
            } else {
                None
            }
        };

        if let Some(i) = i {
            Some(self.src.remove(i))
        } else {
            None
        }
    }

    pub fn resolved(&mut self) -> Option<&mut CommandInteractionDataResolved> {
        self.resolved.as_mut()
    }

    fn get_data(options: &Vec<CommandDataOption>) -> Vec<&CommandDataOption> {
        if let Some(index) = options.iter().position(|item| {
            item.value.kind() == CommandOptionType::SubCommand
                || item.value.kind() == CommandOptionType::SubCommandGroup
        })
        {
            let item = options.get(index).unwrap();
            match &item.value {
                CommandOptionValue::SubCommandGroup(g)
                | CommandOptionValue::SubCommand(g) => Self::get_data(g),
                _ => unreachable!()
            }
        } else {
            options.iter()
                .collect()
        }
    }
}

impl<'a, D> DataIterator<'a, D>
where
    D: Send + Sync
{
    pub async fn named_parse<T>(&mut self, name: &str) -> Result<T, ParseError>
    where
        T: Parse<D>
    {
        let value = self.get(|s| s.name == name);
        if value.is_none() && <T as Parse<D>>::required() {
            Err(ParseError::StructureMismatch(format!("{} not found", name)).into())
        } else {
            Ok(T::parse(
                self.http,
                self.data,
                value.map(|it| &it.value),
                self.resolved())
                .await
                .map_err(|mut err| {
                    if let ParseError::Parsing { argument_name, .. } = &mut err {
                        *argument_name = name.to_string();
                    }
                    err
                })?)
        }
    }
}

impl<'a, D> std::ops::Deref for DataIterator<'a, D> {
    type Target = Vec<&'a CommandDataOption>;

    fn deref(&self) -> &Self::Target {
        &self.src
    }
}

impl<D> std::ops::DerefMut for DataIterator<'_, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.src
    }
}
