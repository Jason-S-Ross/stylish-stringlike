use super::Sliceable;
/// Contains a data structure to allow fast lookup of the value to the left.
use std::borrow::Borrow;
use std::collections::{
    btree_map::{Iter, Range},
    BTreeMap,
};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::ops::{Add, Bound, RangeBounds};
/// Data structure to quickly look up the nearest value smaller than a given value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchTree<V> {
    tree: BTreeMap<usize, V>,
}

impl<V> Default for SearchTree<V> {
    fn default() -> Self {
        Self {
            tree: BTreeMap::<usize, V>::new(),
        }
    }
}

impl<V> SearchTree<V> {
    pub fn new() -> SearchTree<V> {
        SearchTree {
            tree: BTreeMap::<_, _>::new(),
        }
    }
    pub fn contains_key(&self, key: usize) -> bool {
        self.tree.contains_key(&key)
    }
    pub fn range<T, R>(&self, range: R) -> Range<'_, usize, V>
    where
        T: Ord + ?Sized,
        R: RangeBounds<T>,
        usize: Borrow<T> + Ord,
    {
        self.tree.range(range)
    }
    pub fn insert(&mut self, key: usize, value: V) -> Option<V> {
        self.tree.insert(key, value)
    }
    pub fn iter(&self) -> Iter<usize, V> {
        self.tree.iter()
    }
    #[allow(dead_code)]
    pub fn keys(&self) -> Vec<usize> {
        self.tree.keys().cloned().collect()
    }
    pub fn trim(&mut self, max_key: usize) {
        let drop_keys: Vec<_> = self
            .tree
            .iter()
            .filter_map(|(key, _val)| if key > &max_key { Some(key) } else { None })
            .cloned()
            .collect();
        for key in drop_keys {
            self.tree.remove(&key);
        }
    }
    /// Drops keys that have the same value as the previous keys
    pub fn dedup(&mut self)
    where
        V: PartialEq,
    {
        let drop_keys: Vec<_> = self
            .tree
            .iter()
            .zip(self.tree.iter().skip(1))
            .filter_map(|((_first_key, first_val), (second_key, second_val))| {
                if first_val == second_val {
                    Some(second_key)
                } else {
                    None
                }
            })
            .cloned()
            .collect();
        for key in drop_keys {
            self.tree.remove(&key);
        }
    }
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
    /// Copy values in a range from another tree into this tree,
    /// shifting the keys by some amount.
    pub fn copy_with_shift<T, R, S>(
        &mut self,
        from: &SearchTree<V>,
        range: R,
        shift: S,
    ) -> Result<(), Box<dyn Error>>
    where
        V: Clone + PartialEq,
        T: Ord + ?Sized,
        R: RangeBounds<T>,
        usize: Borrow<T> + Ord + TryFrom<S> + Copy,
        S: Add<Output = S> + TryFrom<usize> + Copy,
    {
        let contained_spans = from.range(range);
        for (key, value) in contained_spans {
            // S is a (possibly signed) value
            if let Ok(add_key) = S::try_from(*key) {
                if let Ok(new_key) = usize::try_from(add_key + shift) {
                    self.insert(new_key, value.clone());
                } else {
                    self.insert(*key, value.clone());
                }
            } else {
                return Err(Box::new(ShiftError));
            }
        }
        self.dedup();
        Ok(())
    }
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShiftError;

impl fmt::Display for ShiftError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Error attempting to shift span tree")
    }
}
impl Error for ShiftError {}

impl<V> Sliceable for SearchTree<V>
where
    V: Clone,
{
    fn slice<R>(&self, range: R) -> Option<Self>
    where
        R: std::ops::RangeBounds<usize> + Clone,
    {
        if let Some((zero_key, zero_val)) = self.tree.iter().next() {
            let mut tree: BTreeMap<_, _> = Default::default();
            let (new_zero_key, new_zero_val) = match range.start_bound() {
                Bound::Excluded(x) => {
                    if let Some((_k, v)) = self
                        .tree
                        .range((Bound::Unbounded, Bound::Included(x)))
                        .last()
                    {
                        (*x, v)
                    } else {
                        (*zero_key, zero_val)
                    }
                }
                Bound::Included(x) => {
                    if let Some((_k, v)) = self
                        .tree
                        .range((Bound::Unbounded, Bound::Excluded(x)))
                        .last()
                    {
                        (*x, v)
                    } else {
                        (*zero_key, zero_val)
                    }
                }
                Bound::Unbounded => (*zero_key, zero_val),
            };
            tree.insert(*zero_key, new_zero_val.clone());
            for (key, val) in self.tree.range(range) {
                tree.insert(*key - new_zero_key, val.clone());
            }
            Some(SearchTree { tree })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn dedup() {
        let mut actual = SearchTree::default();
        actual.insert(1, 3);
        actual.insert(2, 3);
        actual.dedup();
        let mut expected = SearchTree::default();
        expected.insert(1, 3);
        assert_eq!(expected, actual);
    }
    #[test]
    fn copy_with_shift_saturating() {
        let mut tree: SearchTree<usize> = Default::default();
        tree.insert(1, 2);
        tree.insert(4, 5);
        let mut actual: SearchTree<_> = Default::default();
        actual.copy_with_shift(&tree, 0.., -2).unwrap();
        let mut expected: SearchTree<usize> = Default::default();
        // since 1 - 2 = -1, coverting back to -1 will fail and we fall back to the original value
        expected.insert(1, 2);
        expected.insert(2, 5);
        assert_eq!(expected, actual);
    }
    #[test]
    fn copy_with_shift() {
        let mut tree: SearchTree<usize> = Default::default();
        tree.insert(2, 2);
        tree.insert(4, 5);
        let mut actual: SearchTree<_> = Default::default();
        actual.copy_with_shift(&tree, 0.., -1).unwrap();
        let mut expected: SearchTree<usize> = Default::default();
        expected.insert(1, 2);
        expected.insert(3, 5);
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_to_zero() {
        let mut tree: SearchTree<usize> = Default::default();
        tree.insert(2, 2);
        tree.insert(4, 5);
        let actual = tree.slice(2..).unwrap();
        let mut expected: SearchTree<usize> = Default::default();
        expected.insert(0, 2);
        expected.insert(2, 5);
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_to_one() {
        let mut tree: SearchTree<usize> = Default::default();
        tree.insert(0, 1);
        tree.insert(2, 2);
        tree.insert(4, 5);
        let actual = tree.slice(1..).unwrap();
        let mut expected: SearchTree<usize> = Default::default();
        expected.insert(0, 1);
        expected.insert(1, 2);
        expected.insert(3, 5);
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_to_three() {
        let mut tree: SearchTree<usize> = Default::default();
        tree.insert(0, 1);
        tree.insert(2, 2);
        tree.insert(4, 5);
        let actual = tree.slice(3..).unwrap();
        let mut expected: SearchTree<usize> = Default::default();
        expected.insert(0, 2);
        expected.insert(1, 5);
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_unbounded_start() {
        let mut tree: SearchTree<usize> = Default::default();
        tree.insert(0, 1);
        tree.insert(2, 2);
        tree.insert(4, 5);
        let actual = tree.slice(..2).unwrap();
        let mut expected: SearchTree<usize> = Default::default();
        expected.insert(0, 1);
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_unbounded_start_inclusive() {
        let mut tree: SearchTree<usize> = Default::default();
        tree.insert(0, 1);
        tree.insert(2, 2);
        tree.insert(4, 5);
        let actual = tree.slice(..=2).unwrap();
        let mut expected: SearchTree<usize> = Default::default();
        expected.insert(0, 1);
        expected.insert(2, 2);
        assert_eq!(expected, actual);
    }
}
