mod sealed {
    pub trait Sealed {}
    impl<T, E> Sealed for Result<T, E> {}
    impl<T> Sealed for Option<T> {}

    pub trait SealedDataOption: Sized {}
    impl SealedDataOption for String {}
    impl SealedDataOption for Option<String> {}
}

/// Defines what items are allowed to be returned from a command function. Since a command
/// function must return a `Result<T, E>`, this trait is only implemented for that type.
pub trait Returnable: sealed::Sealed {
    type Ok;
    type Err;
}

/// Used in the [`after hook`] to determine the inner item, which is required to implement the
/// [returnable] trait.
///
/// [returnable]: self::Returnable
/// [`after hook`]: crate::hook::AfterHook
pub trait Optional: sealed::Sealed {
    type Inner;
}

/// Defines what data types can be used when creating a modal.
pub trait ModalDataOption: sealed::SealedDataOption {
    fn required() -> bool;
    fn parse(item: Option<String>) -> Self;
}

impl<T, E> Returnable for Result<T, E> {
    type Ok = T;
    type Err = E;
}

impl<T> Optional for Option<T> {
    type Inner = T;
}

impl ModalDataOption for Option<String> {
    fn required() -> bool {
        false
    }

    fn parse(item: Option<String>) -> Self {
        if item.as_ref()?.is_empty() {
            None
        } else {
            item
        }
    }
}

impl ModalDataOption for String {
    fn required() -> bool {
        true
    }

    fn parse(item: Option<String>) -> Self {
        item.expect("Item can't be null")
    }
}
