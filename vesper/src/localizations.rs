use std::collections::HashMap;

use crate::{prelude::Framework, command::Command, if_some};
use crate::group::{CommandGroup, GroupParent};

pub type CommandMarker<D, T, E> = LocalizationsProvider<D, T, E, Command<D, T, E>>;
pub type GroupParentMarker<D, T, E> = LocalizationsProvider<D, T, E, GroupParent<D, T, E>>;
pub type CommandGroupMarker<D, T, E> = LocalizationsProvider<D, T, E, CommandGroup<D, T, E>>;

pub trait LocalizationProvider:
    Fn(&Framework<Self::Data, Self::Ok, Self::Err>, &Self::Container) -> HashMap<String, String>
{
    type Data;
    type Ok;
    type Err;
    type Container;
}

//pub(crate) type LocalizationsProvider<D, T, E> = fn(&Framework<D, T, E>, &Command<D, T, E>) -> HashMap<String, String>;
pub(crate) type LocalizationsProvider<D, T, E, C> = fn(&Framework<D, T, E>, &C) -> HashMap<String, String>;

impl<D, T, E, C> LocalizationProvider for LocalizationsProvider<D, T, E, C> {
    type Data = D;
    type Ok = T;
    type Err = E;
    type Container = C;
}

pub struct Localizations<M> {
    map: HashMap<String, String>,
    provider: Option<M>,
}

impl<M> Default for Localizations<M> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            provider: None
        }
    }
}

impl<M> Localizations<M>
where
    M: LocalizationProvider
{
    pub fn get_localizations(
        &self,
        framework: &Framework<M::Data, M::Ok, M::Err>,
        container: &M::Container
    ) -> Option<HashMap<String, String>> {
        let mut localizations = self.map.clone();
        if_some!(&self.provider, |fun| localizations.extend(fun(framework, container)));
        
        if localizations.is_empty() {
            None
        } else {
            Some(localizations)
        }
    }

    pub fn extend<Iter, K, V>(&mut self, iter: Iter) 
    where
        Iter: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: ToString
    {
        self.map.extend(iter.into_iter().map(|(k, v)| (k.to_string(), v.to_string())));
    }

    pub fn set_provider(&mut self, provider: M) {
        self.provider = Some(provider);
    }
}

pub trait Localizable<M: LocalizationProvider> {
    fn name_localizations(&mut self) -> &mut Localizations<M>;
    fn description_localizations(&mut self) -> &mut Localizations<M>;

    fn localized_descriptions<I, K, V>(&mut self, iterator: I) -> &mut Self
        where
            I: IntoIterator<Item = (K, V)>,
            K: ToString,
            V: ToString
    {
        self.name_localizations()
            .extend(iterator);
        self
    }

    fn localized_descriptions_fn(&mut self, fun: M) -> &mut Self {
        self.description_localizations().set_provider(fun);
        self
    }

    fn localized_names<I, K, V>(&mut self, iterator: I) -> &mut Self
        where
            I: IntoIterator<Item = (K, V)>,
            K: ToString,
            V: ToString
    {
        self.name_localizations()
            .extend(iterator);
        self
    }

    fn localized_names_fn(&mut self, fun: M) -> &mut Self {
        self.name_localizations().set_provider(fun);
        self
    }
}
