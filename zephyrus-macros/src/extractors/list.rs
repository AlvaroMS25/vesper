use std::convert::TryInto;
use std::ops::{Deref, DerefMut};

use darling::{FromMeta, Result, export::NestedMeta};

pub struct List<T> {
    pub inner: Vec<T>
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self {
            inner: Vec::new()
        }
    }
}

impl<T> Deref for List<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for List<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: FromMeta> FromMeta for List<T> {
    fn from_list(items: &[NestedMeta]) -> Result<Self> {
        let items = items.iter()
            .map(FromMeta::from_nested_meta)
            .collect::<Result<Vec<T>>>()?;
        
        Ok(Self {
            inner: items
        })
    }
}

pub struct FixedList<const SIZE: usize, T> {
    pub inner: [T; SIZE]
}

impl<const SIZE: usize, T: FromMeta> FromMeta for FixedList<SIZE, T> {
    fn from_list(items: &[NestedMeta]) -> Result<Self> {
        if items.len() > SIZE {
            Err(darling::Error::too_many_items(SIZE))?;
        } else if items.len() < SIZE {
            Err(darling::Error::too_few_items(SIZE))?;
        }

        let items = items.iter()
            .map(FromMeta::from_nested_meta)
            .collect::<Result<Vec<T>>>()?;

        fn to_array<T, const S: usize>(vec: Vec<T>) -> Result<[T; S]> {
            vec.try_into()
                .map_err(|_| darling::Error::custom("Failed to construct fixed list"))
        }

        Ok(Self {
            inner: to_array(items)?
        })
    }
}
