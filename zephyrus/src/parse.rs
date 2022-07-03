use crate::{builder::WrappedClient, twilight_exports::*};
use async_trait::async_trait;
use std::error::Error;

#[doc(hidden)]
#[derive(Debug)]
// Generic error used by the framework.
pub struct GenericParsingError(&'static str);

impl GenericParsingError {
    pub fn new(message: &'static str) -> ParseError {
        ParseError::Parse(Box::new(Self(message)) as Box<_>)
    }
}

impl std::fmt::Display for GenericParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for GenericParsingError {}

/// The core trait of this framework, it is used to parse all command arguments
#[async_trait]
pub trait Parse<T: Send + Sync>: Sized {
    /// Adds the possible choices to the argument, this function is usually implemented by the
    /// derive macro, but can be overridden manually.
    fn add_choices() -> Box<dyn Fn() -> Option<Vec<CommandOptionChoice>> + Send + Sync> {
        Box::new(|| None)
    }

    /// Parses the option into the argument.
    async fn parse(
        _http_client: &WrappedClient,
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
