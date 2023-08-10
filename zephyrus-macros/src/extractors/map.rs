use std::{collections::HashMap, hash::Hash, ops::{Deref, DerefMut}};

use darling::{FromMeta, Result, export::NestedMeta, error::Accumulator, Error};
use quote::ToTokens;
use syn::Meta;

pub struct Map<K, V> {
    inner: HashMap<K, V>
}

pub struct Tuple<'a, K, V>(&'a K, &'a V);

impl<K, V> FromMeta for Map<K, V>
where
    K: FromMeta + Hash + Eq + ToString,
    V: FromMeta
{
    fn from_list(items: &[NestedMeta]) -> Result<Self> {
        let mut accumulator = Accumulator::default();
        let mut inner = HashMap::with_capacity(items.len());

        for meta in items {
            match meta {
                NestedMeta::Meta(inner_meta) => {
                    let key = accumulator
                        .handle(<K as FromMeta>::from_meta(&Meta::Path(inner_meta.path().clone())));
                    let value = accumulator
                        .handle(FromMeta::from_meta(inner_meta));

                    if let (Some(k), Some(v)) = (key, value) {
                        if inner.contains_key(&k) {
                            accumulator.handle::<()>(
                                Err(Error::duplicate_field(&k.to_string()).with_span(&inner_meta))
                            );
                        } else {
                            inner.insert(k, v);
                        }
                    }
                },
                other => {
                    accumulator.handle::<()>(
                        Err(Error::unsupported_format("Literal").with_span(&other))
                    );
                }
            }
        }

        accumulator.finish()?;
        Ok(Map {
            inner
        })
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
    pub fn pairs(&self) -> Vec<Tuple<K, V>> {
        todo!()
    }
}

impl<'a, K, V> ToTokens for Tuple<'a, K, V>
where
    K: ToTokens,
    V: ToTokens
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let k = self.0;
        let v = self.1;
        tokens.extend(quote::quote!((#k, #v)))
    }
}
