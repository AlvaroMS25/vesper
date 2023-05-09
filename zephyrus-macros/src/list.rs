use darling::{FromMeta, Result, export::NestedMeta};

pub struct List<T> {
    pub inner: Vec<T>
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
