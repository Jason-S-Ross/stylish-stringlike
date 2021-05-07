use std::ops::{Index, Range};
use std::slice::SliceIndex;
/// Implements string-like operations for string-like things
pub trait StringLike<T>: Index<T> + ToString
where
    T: SliceIndex<Range<usize>>,
{
    fn contains<P, Q>(&self, pat: P) -> bool
    where
        P: StringLike<Q>,
        Q: SliceIndex<Range<usize>>;
    // fn starts_with<'a, P>(&'a self, pat: P) -> bool
    // where P: StringLike<Q>, Q: SliceIndex<str>;
    // fn ends_with<'a, P>(&'a self, pat: P) -> bool
    // where P: StringLike<T>;
    // fn find<'a, P>(&'a self, pat: P) -> Option<usize>
    // where P: StringLike<T>;
    // fn rfind<'a, P>(&'a self, pat: P) -> Option<usize>
    // where P: StringLike<T>;
    // fn split<'a, P>(&'a self, pat: P) -> Vec<P>
    // where P: StringLike<T>;
    // fn split_inclusive<'a, P>(&'a self, pat: P) -> Vec<P>
    // where P: StringLike<T>;
}
