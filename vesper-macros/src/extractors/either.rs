use darling::{FromMeta, Result, export::NestedMeta, error::Accumulator};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream, discouraged::Speculative};

use super::FixedList;

macro_rules! try_both {
    (@inner $fun:ident, [$($args:ident),* $(,)?]) => {{
        let mut accumulator = Accumulator::default();
        if let Some(parsed) = accumulator.handle(A::$fun($($args),*)) {
            accumulator.finish().unwrap(); // Since we are here, we know the accumulator is empty
            Ok(Either::Left(parsed))
        } else {
            let parsed = accumulator.handle(B::$fun($($args),*));
            if let Some(parsed) = parsed {
                let _ = accumulator.finish(); // Discard previous error.
                Ok(Either::Right(parsed))
            } else {
                // If we are here, both parsing failed, so the accumulator will return an error
                // and exit the function.
                accumulator.finish()?;
                unreachable!()
            }
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

impl<A: Default, B> Default for Either<A, B> {
    fn default() -> Self {
        Self::Left(A::default())
    }
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

impl<A, B> Parse for Either<A, B>
where
    A: Parse,
    B: Parse
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fork = input.fork();

        if let Ok(parsed) = fork.parse::<A>() {
            input.advance_to(&fork);

            Ok(Either::Left(parsed))
        } else {
            Ok(Either::Right(input.parse::<B>()?))
        }
    }
}

impl<A, B, I> Either<A, B>
where
    for<'a> &'a A: IntoIterator<Item = I>,
    for<'a> &'a B: IntoIterator<Item = I>
{
    #[allow(dead_code)]
    pub fn iter_ref<'a>(&'a self) -> Box<dyn Iterator<Item = I> + 'a> {
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
    #[allow(dead_code)]
    pub fn iter_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = I> + 'a> {
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
    #[allow(dead_code)]
    pub fn into_iter(self) -> Box<dyn Iterator<Item = I>> {
        match self {
            Either::Left(a) => Box::new(a.into_iter()),
            Either::Right(b) => Box::new(b.into_iter())
        }
    }
}

impl<A> Either<A, FixedList<1, A>> {
    pub fn inner(&self) -> &A {
        match self {
            Self::Left(a) => a,
            Self::Right(list) => &list.inner[0]
        }
    }
}

impl<A, B> Either<A, B> {
    pub fn map_1<T, F1, F2, R>(&self, data: &mut T, mut f1: F1, mut f2: F2) -> R
    where
        F1: FnMut(&mut T, &A) -> R,
        F2: FnMut(&mut T, &B) -> R
    {
        match self {
            Self::Left(a) => f1(data, a),
            Self::Right(b) => f2(data, b)
        }
    }
}

impl<A: ToTokens, B: ToTokens> ToTokens for Either<A, B> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Left(a) => <A as ToTokens>::to_tokens(a, tokens),
            Self::Right(b) => <B as ToTokens>::to_tokens(b, tokens)
        }
    }
}
