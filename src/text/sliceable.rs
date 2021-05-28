use std::ops::{Bound, RangeBounds};
/// Provides function for slicing a text object on byte index (like [`str::get`])
pub trait Sliceable {
    /// Slice an underlying text object by bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use stylish_stringlike::text::Sliceable;
    /// let foo = "foobar";
    /// assert_eq!(foo.get(1..4), foo.slice(1..4));
    /// ```
    fn slice<R>(&self, range: R) -> Option<Self>
    where
        R: std::ops::RangeBounds<usize> + Clone,
        Self: Sized;
}

impl<'a> Sliceable for &'a str {
    fn slice<R>(&self, range: R) -> Option<Self>
    where
        R: RangeBounds<usize> + Clone,
        Self: Sized,
    {
        match (range.start_bound(), range.end_bound()) {
            (Bound::Unbounded, Bound::Unbounded) => self.get(..),
            (Bound::Unbounded, Bound::Excluded(e)) => self.get(..*e),
            (Bound::Unbounded, Bound::Included(e)) => self.get(..=*e),
            (Bound::Excluded(s), Bound::Unbounded) => self.get((*s + 1)..),
            (Bound::Excluded(s), Bound::Excluded(e)) => self.get((*s + 1)..*e),
            (Bound::Excluded(s), Bound::Included(e)) => self.get((*s + 1)..=*e),
            (Bound::Included(s), Bound::Unbounded) => self.get(*s..),
            (Bound::Included(s), Bound::Excluded(e)) => self.get(*s..*e),
            (Bound::Included(s), Bound::Included(e)) => self.get(*s..=*e),
        }
    }
}

impl Sliceable for String {
    fn slice<R>(&self, range: R) -> Option<Self>
    where
        R: RangeBounds<usize> + Clone,
        Self: Sized,
    {
        self.as_str().slice(range).map(String::from)
    }
}
