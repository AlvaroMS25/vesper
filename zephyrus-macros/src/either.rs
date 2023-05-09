use darling::{FromMeta, Result, export::{NestedMeta}, error::Accumulator};

macro_rules! try_both {
    (@inner $fun:ident, [$($args:ident),* $(,)?]) => {{
        let mut accumulator = Accumulator::default();
        if let Some(parsed) = accumulator.handle(A::$fun($($args),*)) {
            accumulator.finish().unwrap(); // Since we are here, we know the accumulator is empty
            Ok(Either::Left(parsed))
        } else {
            let parsed = accumulator.handle(B::$fun($($args),*));
            accumulator.finish()?;
            // If we are here, there were no errors in the accumulator and B was parsed.
            Ok(Either::Right(parsed.unwrap()))
        }
    }};
    ($fun:ident, $($args:ident),* $(,)?) => {
        try_both!(@inner $fun, [$($args),*])
    };
    ($fun:ident) => {
        try_both!(@inner $fun, [])
    }
}

#[derive(Clone)]
pub enum Either<A, B> {
    Left(A),
    Right(B)
}

impl<A: FromMeta, B: FromMeta> FromMeta for Either<A, B> {
    fn from_nested_meta(item: &NestedMeta) -> Result<Self> {
        try_both!(from_nested_meta, item)
    }

    fn from_meta(item: &syn::Meta) -> Result<Self> {
        try_both!(from_meta, item)
    }

    fn from_none() -> Option<Self> {
        let a = A::from_none();
        a
            .map(Either::Left)
            .or_else(|| B::from_none().map(Either::Right))
    }

    fn from_word() -> Result<Self> {
        try_both!(from_word)
    }

    fn from_list(items: &[NestedMeta]) -> Result<Self> {
        try_both!(from_list, items)
    }

    fn from_value(value: &syn::Lit) -> Result<Self> {
        try_both!(from_value, value)
    }

    fn from_expr(expr: &syn::Expr) -> Result<Self> {
        try_both!(from_expr, expr)
    }

    fn from_char(value: char) -> Result<Self> {
        try_both!(from_char, value)
    }

    fn from_string(value: &str) -> Result<Self> {
        try_both!(from_string, value)
    }

    fn from_bool(value: bool) -> Result<Self> {
        try_both!(from_bool, value)
    }
}

impl<A, B, I> Either<A, B>
where
    for<'a> &'a A: IntoIterator<Item = I>,
    for<'a> &'a B: IntoIterator<Item = I>
{
    fn iter_ref<'a>(&'a self) -> Box<dyn Iterator<Item = I> + 'a> {
        match &self {
            Either::Left(a) => Box::new(a.into_iter()),
            Either::Right(b) => Box::new(b.into_iter())
        }
    }
}

impl<A, B, I> Either<A, B>
where
    for<'a> &'a mut A: IntoIterator<Item = I>,
    for<'a> &'a mut B: IntoIterator<Item = I>
{
    fn iter_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = I> + 'a> {
        match self {
            Either::Left(a) => Box::new(a.into_iter()),
            Either::Right(b) => Box::new(b.into_iter())
        }
    }
}

impl<A, B, I> Either<A, B>
where
    A: IntoIterator<Item = I> + 'static,
    B: IntoIterator<Item = I> + 'static
{
    fn into_iter(self) -> Box<dyn Iterator<Item = I>> {
        match self {
            Either::Left(a) => Box::new(a.into_iter()),
            Either::Right(b) => Box::new(b.into_iter())
        }
    }
}
