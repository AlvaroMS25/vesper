use crate::context::SlashContext;
use crate::twilight_exports::{InteractionData, CommandDataOption, CommandInteractionDataResolved};

/// An iterator used to iterate through slash command options.
pub struct DataIterator<'a> {
    src: Vec<&'a CommandDataOption>,
    resolved: &'a mut Option<CommandInteractionDataResolved>
}

impl<'a> DataIterator<'a> {
    /// Creates a new [iterator](self::DataIterator) at the given source.
    pub fn new<T>(ctx: &'a SlashContext<'a, T>) -> Self {
        let data = match ctx.interaction_mut().data.as_mut().unwrap() {
            InteractionData::ApplicationCommand(data) => data,
            _ => unreachable!()
        };

        let options = data
            .options
            .iter()
            .collect::<Vec<_>>();

        Self {
            src: options,
            resolved: &mut data.resolved
        }
    }
}

impl<'a> DataIterator<'a> {
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
}

impl<'a> std::ops::Deref for DataIterator<'a> {
    type Target = Vec<&'a CommandDataOption>;

    fn deref(&self) -> &Self::Target {
        &self.src
    }
}

impl std::ops::DerefMut for DataIterator<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.src
    }
}
