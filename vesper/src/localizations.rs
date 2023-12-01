use std::collections::HashMap;

use crate::{prelude::Framework, command::Command, if_some};

pub struct Localizations<D, T, E> {
    pub map: HashMap<String, String>,
    pub provider: Option<fn(&Framework<D, T, E>, &Command<D, T, E>) -> HashMap<String, String>>,
}

impl<D, T, E> Default for Localizations<D, T, E> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            provider: None
        }
    }
}

impl<D, T, E> Localizations<D, T, E> {
    pub fn get_localizations(
        &self,
        framework: &Framework<D, T, E>,
        command: &Command<D, T, E>
    ) -> HashMap<String, String> {
        let mut localizations = self.map.clone();
        if_some!(&self.provider, |fun| localizations.extend(fun(framework, command)));
        localizations
    }

    pub fn extend<Iter, K, V>(&mut self, iter: Iter) 
    where
        Iter: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: ToString
    {
        self.map.extend(iter.into_iter().map(|(k, v)| (k.to_string(), v.to_string())));
    }
}
