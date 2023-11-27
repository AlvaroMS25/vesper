use std::collections::HashMap;

use crate::{prelude::Framework, command::Command};

pub struct Localizations<D, T, E> {
    pub map: HashMap<String, String>,
    pub r#fn: fn(&Framework<D, T, E>, &Command<D, T, E>) -> HashMap<String, String>
}