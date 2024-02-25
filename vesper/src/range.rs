use std::any::type_name;
use crate::prelude::*;
use crate::twilight_exports::*;
use crate::parse_impl::error;
use std::ops::{Deref, DerefMut};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};


mod sealed {
    use super::*;

    /// A trait used to specify the values [range](super::Range) can take.
    pub trait Number: Copy + Debug + Display {
        fn as_i64(&self) -> i64;
    }

    macro_rules! number {
        ($($t:ty),* $(,)?) => {
            $(
                impl Number for $t {
                    fn as_i64(&self) -> i64 {
                        *self as i64
                    }
                }
            )*
        };
    }

    number![i8, i16, i32, i64, isize, u8, u16, u32, u64, usize];
}

use sealed::Number;

/// A range-like type used to constraint the input provided by the user. This is equivalent to
/// using a [RangeInclusive], but implements the [parse] trait.
///
/// [RangeInclusive]: std::ops::RangeInclusive
/// [parse]: Parse
#[derive(Copy, Clone)]
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
    async fn parse(
        http_client: &WrappedClient,
        data: &T,
        value: Option<&CommandOptionValue>,
        resolved: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        let value = E::parse(http_client, data, value, resolved).await?;

        let v = value.as_i64();

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

    fn modify_option(option: &mut CommandOption) {
        use twilight_model::application::command::CommandOptionValue;
        option.max_value = Some(CommandOptionValue::Integer(END));
        option.min_value = Some(CommandOptionValue::Integer(START));
    }
}

impl<T: Number, const START: i64, const END: i64> Debug for Range<T, START, END> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Range<{}, {}, {}>({})", type_name::<T>(), START, END, self.0)
    }
}
