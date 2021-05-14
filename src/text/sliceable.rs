use std::slice::SliceIndex;
pub trait Sliceable<'a, R, S: ?Sized> {
    type Output;
    type Error;
    fn slice(&'a self, range: R) -> Result<Self::Output, Self::Error>
    where
        S: 'a,
        R: SliceIndex<S, Output = S>;
}
