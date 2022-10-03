use crate::{argument::ArgumentLimits, builder::WrappedClient, twilight_exports::*};
use async_trait::async_trait;
use std::error::Error;

/// The core trait of this framework, it is used to parse all command arguments
#[async_trait]
pub trait Parse<T: Send + Sync>: Sized {
    /// Parses the option into the argument.
    async fn parse(
        _http_client: &WrappedClient,
        _data: &T,
        _value: Option<&CommandOptionValue>,
    ) -> Result<Self, ParseError>;

    /// Returns the option type this argument has.
    fn kind() -> CommandOptionType;

    /// Sets if the argument is required, by default is true.
    fn required() -> bool {
        true
    }

    /// Adds the possible choices to the argument, this function is usually implemented by the
    /// derive macro, but can be overridden manually.
    fn choices() -> Option<Vec<CommandOptionChoice>> {
        None
    }

    fn limits() -> Option<ArgumentLimits> {
        None
    }
}

/// The errors which can be returned from [Parse](self::Parse) [parse](self::Parse::parse) function.
#[derive(Debug)]
pub enum ParseError {
    StructureMismatch(String),
    Parsing {
        argument_name: String,
        required: bool,
        type_: String,
        error: String
    },
    Other(Box<dyn Error + Send + Sync>),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StructureMismatch(why) => write!(f, "Structure mismatch: {}", why),
            Self::Parsing { argument_name, required, type_, error } => {
                write!(f, "Failed to parse {}({}required {}): {}", argument_name, {
                    if !required {
                        "not "
                    } else {
                        ""
                    }
                }, type_, error)
            }
            Self::Other(why) => write!(f, "Other: {}", why),
        }
    }
}
impl Error for ParseError {}

impl From<Box<dyn Error + Send + Sync>> for ParseError {
    fn from(e: Box<dyn Error + Send + Sync>) -> Self {
        Self::Other(e)
    }
}

impl From<&'static str> for ParseError {
    fn from(why: &'static str) -> Self {
        Self::StructureMismatch(why.to_string())
    }
}
