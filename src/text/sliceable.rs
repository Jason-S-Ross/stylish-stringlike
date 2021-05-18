use std::ops::{Bound, Deref, RangeBounds};
/// Byte-indexed sliceables
pub trait Sliceable<'a> {
    type Output;
    type Index;
    fn slice<R>(&'a self, range: R) -> Option<Self::Output>
    where
        R: std::ops::RangeBounds<Self::Index> + Clone;
}

pub fn slice_string<'a, R, T>(string: &'a T, range: R) -> Option<&'a str>
where
    R: RangeBounds<usize>,
    T: Deref<Target = str> + 'a,
{
    match (range.start_bound(), range.end_bound()) {
        (Bound::Unbounded, Bound::Unbounded) => string.get(..),
        (Bound::Unbounded, Bound::Excluded(e)) => string.get(..*e),
        (Bound::Unbounded, Bound::Included(e)) => string.get(..=*e),
        (Bound::Excluded(_), _) => unreachable!("Bound found with excluded start"),
        (Bound::Included(s), Bound::Unbounded) => string.get(*s..),
        (Bound::Included(s), Bound::Excluded(e)) => string.get(*s..*e),
        (Bound::Included(s), Bound::Included(e)) => string.get(*s..=*e),
    }
}
