mod sealed {
    pub trait Sealed {}
    impl<T, E> Sealed for Result<T, E> {}
}

pub trait Returnable: sealed::Sealed {
    type Ok;
    type Err;
}

impl<T, E> Returnable for Result<T, E> {
    type Ok = T;
    type Err = E;
}
