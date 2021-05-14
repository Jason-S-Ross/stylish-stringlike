/// Contains a data structure to allow fast lookup of the value to the left.
use std::borrow::Borrow;
use std::collections::btree_map::Iter;
use std::collections::btree_map::Range;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::ops::{Add, RangeBounds};
/// Data structure to quickly look up the nearest value smaller than a given value.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SearchTree<K, V>
where
    K: Ord,
{
    tree: BTreeMap<K, V>,
}
impl<K, V> SearchTree<K, V>
where
    K: Ord,
{
    pub fn new() -> SearchTree<K, V>
    where
        K: Ord,
    {
        SearchTree {
            tree: BTreeMap::<K, V>::new(),
        }
    }
    pub fn range<T, R>(&self, range: R) -> Range<'_, K, V>
    where
        T: Ord + ?Sized,
        R: RangeBounds<T>,
        K: Borrow<T> + Ord,
    {
        self.tree.range(range)
    }
    pub fn search_left<T>(&self, key: &T) -> Option<&V>
    where
        T: Ord,
        K: Borrow<T> + Ord,
    {
        if let Some(ref v) = self.tree.get(key) {
            Some(v)
        } else if let Some((_last_key, ref v)) = self.tree.range(..key).last() {
            Some(v)
        } else {
            None
        }
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        self.tree.insert(key, value)
    }
    pub fn iter(&self) -> Iter<K, V> {
        self.tree.iter()
    }
    #[allow(dead_code)]
    pub(super) fn keys(&self) -> Vec<K>
    where
        K: Clone,
    {
        self.tree.keys().cloned().collect()
    }
    /// Drops keys that have the same value as the previous keys
    fn dedup(&mut self)
    where
        V: PartialEq,
        K: Clone,
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
    /// Copy values in a range from another tree into this tree,
    /// shifting the keys by some amount.
    pub(super) fn copy_with_shift<T, R, S>(
        &mut self,
        from: &SearchTree<K, V>,
        range: R,
        shift: S,
    ) -> Result<(), ()>
    where
        V: Clone + PartialEq,
        T: Ord + ?Sized,
        R: RangeBounds<T>,
        K: Borrow<T> + Ord + TryFrom<S> + Copy,
        S: Add<Output = S> + TryFrom<K> + Copy,
    {
        let contained_spans = from.range(range);
        for (key, value) in contained_spans {
            if let Ok(add_key) = S::try_from(*key) {
                if let Ok(new_key) = K::try_from(add_key + shift) {
                    self.insert(new_key, value.clone());
                } else {
                    self.insert(*key, value.clone());
                }
            } else {
                return Err(());
            }
        }
        self.dedup();
        Ok(())
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
    fn search_left() {
        let mut tree = SearchTree::default();
        tree.insert(1, 2);
        tree.insert(3, 4);
        let actual = tree.search_left(&2);
        let expected = Some(&2);
        assert_eq!(expected, actual);
    }
    #[test]
    fn search_on() {
        let mut tree = SearchTree::default();
        tree.insert(1, 2);
        tree.insert(3, 4);
        let actual = tree.search_left(&1);
        let expected = Some(&2);
        assert_eq!(expected, actual);
    }
    #[test]
    fn search_missing() {
        let mut tree = SearchTree::default();
        tree.insert(1, 2);
        tree.insert(3, 4);
        let actual = tree.search_left(&0);
        let expected = None;
        assert_eq!(expected, actual);
    }
    #[test]
    fn copy_with_shift_saturating() {
        let mut tree: SearchTree<usize, usize> = Default::default();
        tree.insert(1, 2);
        tree.insert(4, 5);
        let mut actual: SearchTree<_, _> = Default::default();
        actual.copy_with_shift(&tree, 0.., -2).unwrap();
        let mut expected: SearchTree<usize, usize> = Default::default();
        // since 1 - 2 = -1, coverting back to -1 will fail and we fall back to the original value
        expected.insert(1, 2);
        expected.insert(2, 5);
        assert_eq!(expected, actual);
    }
    #[test]
    fn copy_with_shift() {
        let mut tree: SearchTree<usize, usize> = Default::default();
        tree.insert(2, 2);
        tree.insert(4, 5);
        let mut actual: SearchTree<_, _> = Default::default();
        actual.copy_with_shift(&tree, 0.., -1).unwrap();
        let mut expected: SearchTree<usize, usize> = Default::default();
        expected.insert(1, 2);
        expected.insert(3, 5);
        assert_eq!(expected, actual);
    }
    #[test]
    fn copy_with_shift_fail() {
        let mut tree: SearchTree<isize, usize> = Default::default();
        tree.insert(-2, 2);
        tree.insert(4, 5);
        let mut actual: SearchTree<isize, _> = Default::default();
        let offset: usize = 1;
        // this will fail since we can't cast the offset to the type isize
        let res = actual.copy_with_shift(&tree, -4..5, offset);
        assert_eq!(Err(()), res);
    }
}
