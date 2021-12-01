use crate::{iter::DataIterator, twilight_exports::*};
use async_trait::async_trait;
use std::error::Error;

/// The core trait of this framework, it is used to parse all command arguments
#[async_trait]
pub trait Parse<T: Send + Sync + 'static>: Sized {
    /// Parses an argument by the option name, this is the entry point of this trait and should
    /// never be overridden.
    async fn named_parse(
        name: &'static str,
        http_client: &Client,
        data: &T,
        iterator: &mut DataIterator,
    ) -> Result<Self, ParseError> {
        let out = iterator.get(|s| s.name == name);

        if let Some(o) = out {
            Self::parse(http_client, data, Some(&o.value)).await
        } else {
            if Self::is_required() {
                Err(ParseError::StructureMismatch(format!(
                    "{} not found",
                    name
                )))
            } else {
                Self::parse(http_client, data, None).await
            }
        }
    }

    /// Adds the possible choices to the argument, this function is usually implemented by the
    /// derive macro, but can be overridden manually.
    fn add_choices() -> Box<dyn Fn() -> Option<Vec<CommandOptionChoice>> + Send> {
        Box::new(|| None)
    }

    /// Parses the option into the argument.
    async fn parse(
        _http_client: &Client,
        _data: &T,
        _value: Option<&CommandOptionValue>,
    ) -> Result<Self, ParseError>;

    /// Sets if the argument is required, by default is true.
    fn is_required() -> bool {
        true
    }

    /// Returns the option type this argument has.
    fn option_type() -> CommandOptionType;
}

/// The errors which can be returned from [Parse](self::Parse) [parse](self::Parse::parse) function.
#[derive(Debug)]
pub enum ParseError {
    StructureMismatch(String),
    Parse(Box<dyn Error + Send + Sync>),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StructureMismatch(why) => write!(f, "Structure mismatch: {}", why),
            Self::Parse(why) => write!(f, "Failed parsing an argument: {}", why),
        }
    }
}
impl Error for ParseError {}

impl From<Box<dyn Error + Send + Sync>> for ParseError {
    fn from(e: Box<dyn Error + Send + Sync>) -> Self {
        Self::Parse(e)
    }
}

impl From<&'static str> for ParseError {
    fn from(why: &'static str) -> Self {
        Self::StructureMismatch(why.to_string())
    }
}
