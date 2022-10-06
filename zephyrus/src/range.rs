use std::any::type_name;
use crate::prelude::*;
use crate::twilight_exports::*;
use crate::parse_impl::error;
use std::ops::{Deref, DerefMut};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};


mod sealed {
    use super::*;

    /// A trait used to specify the values [range](super::Range) can take.
    pub trait Number: Copy + Debug + Display {}

    macro_rules! number {
        ($($t:ty),* $(,)?) => {
            $(
                impl Number for $t {}
            )*
        };
    }

    number![i8, i16, i32, i64, isize, u8, u16, u32, u64, usize];
}

use sealed::Number;

/// A range-like type used to constraint the input provided by the user.
pub struct Range<T: Number, const START: i64, const END: i64>(T);

impl<T: Number, const START: i64, const END: i64> Deref for Range<T, START, END> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Number, const START: i64, const END: i64> DerefMut for Range<T, START, END> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait]
impl<T, E, const START: i64, const END: i64> Parse<T> for Range<E, START, END>
    where
        T: Send + Sync,
        E: Parse<T> + Number
{
    async fn parse(http_client: &WrappedClient, data: &T, value: Option<&CommandOptionValue>) -> Result<Self, ParseError> {
        let value = E::parse(http_client, data, value).await?;

        // SAFETY: The maximum value allowed by discord can be represented as an i64,
        // so casting to it won't lead to losing any data.
        let v = unsafe { *(&value as *const E as *const i64) };

        if v < START || v > END {
            return Err(error(
                &format!("Range<{}, {}, {}>", type_name::<E>(), START, END),
                true,
                "Input out of range"
            ));
        }

        Ok(Self(value))
    }

    fn kind() -> CommandOptionType {
        E::kind()
    }

    fn limits() -> Option<ArgumentLimits> {
        use twilight_model::application::command::CommandOptionValue;
        Some(ArgumentLimits {
            min: Some(CommandOptionValue::Integer(START)),
            max: Some(CommandOptionValue::Integer(END))
        })
    }
}

impl<T: Number, const START: i64, const END: i64> Debug for Range<T, START, END> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Range<{}, {}, {}>({})", type_name::<T>(), START, END, self.0)
    }
}
