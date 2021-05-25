use std::ops::{Bound, Deref, RangeBounds};
/// Provides function for slicing a text object on byte index (like [`String::get`])
pub trait Sliceable<'a> {
    /// Slice an underlying text object by bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use stylish_stringlike::text::Sliceable;
    /// let foo = "foobar";
    /// assert_eq!(foo.get(1..4), foo.slice(1..4));
    /// ```
    fn slice<R>(&'a self, range: R) -> Option<Self>
    where
        R: std::ops::RangeBounds<usize> + Clone,
        Self: Sized;
}

impl<'a> Sliceable<'a> for &'a str {
    fn slice<R>(&'a self, range: R) -> Option<Self>
    where
        R: std::ops::RangeBounds<usize> + Clone,
        Self: Sized,
    {
        slice_string(self, range)
    }
}

impl<'a> Sliceable<'a> for String {
    fn slice<R>(&'a self, range: R) -> Option<Self>
    where
        R: std::ops::RangeBounds<usize> + Clone,
        Self: Sized,
    {
        slice_string(&self.as_str(), range).map(String::from)
    }
}

/// Adapter function to convert generic ranges into ranges that string slices will accept.
pub(crate) fn slice_string<'a, R, T>(string: &'a T, range: R) -> Option<&'a str>
where
    R: RangeBounds<usize>,
    T: Deref<Target = str> + 'a,
{
    match (range.start_bound(), range.end_bound()) {
        (Bound::Unbounded, Bound::Unbounded) => string.get(..),
        (Bound::Unbounded, Bound::Excluded(e)) => string.get(..*e),
        (Bound::Unbounded, Bound::Included(e)) => string.get(..=*e),
        (Bound::Excluded(s), Bound::Unbounded) => string.get((*s + 1)..),
        (Bound::Excluded(s), Bound::Excluded(e)) => string.get((*s + 1)..*e),
        (Bound::Excluded(s), Bound::Included(e)) => string.get((*s + 1)..=*e),
        (Bound::Included(s), Bound::Unbounded) => string.get(*s..),
        (Bound::Included(s), Bound::Excluded(e)) => string.get(*s..*e),
        (Bound::Included(s), Bound::Included(e)) => string.get(*s..=*e),
    }
}
