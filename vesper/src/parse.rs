use crate::{builder::WrappedClient, twilight_exports::*};
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
        _resolved: Option<&mut InteractionDataResolved>
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

    fn modify_option(_option: &mut CommandOption) {}
}

/// The errors which can be returned from [Parse](self::Parse) [parse](self::Parse::parse) function.
#[derive(Debug)]
pub enum ParseError {
    /// The command arguments does not match with the framework ones.
    StructureMismatch(String),
    /// An argument failed parsing.
    Parsing {
        /// The name of the argument that failed to parse.
        argument_name: String,
        /// Whether if the argument is required or not-
        required: bool,
        /// The type of the argument.
        argument_type: String,
        /// The error message as a string.
        error: String
    },
    /// Other error occurred.
    Other(Box<dyn Error + Send + Sync>),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StructureMismatch(why) => write!(f, "Structure mismatch: {}", why),
            Self::Parsing { argument_name, required, argument_type, error } => {
                write!(f, "Failed to parse {}({}required {}): {}", argument_name, {
                    if !required {
                        "not "
                    } else {
                        ""
                    }
                }, argument_type, error)
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
