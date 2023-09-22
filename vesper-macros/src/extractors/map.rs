use std::{collections::HashMap, hash::Hash, ops::{Deref, DerefMut}};

use darling::{FromMeta, Result};
use syn::{Meta, punctuated::Punctuated, Token, parse::{Parse, Parser}};

use crate::extractors::Tuple2;

pub struct Map<K, V> {
    inner: HashMap<K, V>
}

impl<K, V> FromMeta for Map<K, V>
where
    K: Parse + Eq + Hash,
    V: Parse
{
    fn from_meta(item: &Meta) -> Result<Self> {
        match item {
            Meta::List(inner) => {
                let items = Punctuated::<Tuple2<K, V, Token![=]>, Token![,]>::parse_terminated
                    .parse2(inner.tokens.clone())?;

                let mut map = HashMap::with_capacity(items.len()); 
                for item in items {
                    map.insert(item.0, item.1);
                }

                Ok(Map {
                    inner: map
                })
            },
            _ => Err(darling::Error::unsupported_format("Item list").with_span(&item))
        }
    }
}

impl<K, V> Deref for Map<K, V> {
    type Target = HashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K, V> DerefMut for Map<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<K, V> Map<K, V> {
    pub fn pairs(&self) -> Vec<Tuple2<&K, &V>> {
        self.inner.iter().map(|(k, v)| Tuple2::new(k, v)).collect()
    }
}
