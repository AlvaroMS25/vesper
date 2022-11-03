mod sealed {
    pub trait Sealed {}
    impl<T, E> Sealed for Result<T, E> {}
    impl<T: Sealed> Sealed for Option<T> {}
}

pub trait Returnable: sealed::Sealed {
    type Ok;
    type Err;
}

pub trait Optional: sealed::Sealed {
    type Inner;
}

impl<T, E> Returnable for Result<T, E> {
    type Ok = T;
    type Err = E;
}

impl<T> Optional for Option<T>
