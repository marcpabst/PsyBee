//  get a Box<T> from T
pub trait IntoBoxed<T> {
    fn into_boxed(self) -> Box<T>;
}

impl<T> IntoBoxed<T> for T {
    fn into_boxed(self) -> Box<T> {
        Box::new(self)
    }
}
