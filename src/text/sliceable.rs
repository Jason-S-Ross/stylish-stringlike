use std::slice::SliceIndex;
pub trait Sliceable<'a> {
    type Output;
    type Target: ?Sized;
    type Index;
    fn slice<R>(&'a self, range: R) -> Self::Output
    where
        R: SliceIndex<Self::Target, Output = Self::Target>
            + std::ops::RangeBounds<Self::Index>
            + Clone;
}
