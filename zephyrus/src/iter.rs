use crate::twilight_exports::CommandDataOption;

/// An iterator used to iterate through slash command options.
pub struct DataIterator<'a> {
    src: Vec<&'a CommandDataOption>,
}

impl<'a> DataIterator<'a> {
    /// Creates a new [iterator](self::DataIterator) at the given source.
    pub fn new(src: Vec<&'a CommandDataOption>) -> Self {
        Self { src }
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
