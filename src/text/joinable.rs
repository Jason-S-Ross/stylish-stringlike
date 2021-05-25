pub(crate) trait Joinable<T> {
    type Output: Sized;
    fn join(&self, other: &T) -> Self::Output;
}
