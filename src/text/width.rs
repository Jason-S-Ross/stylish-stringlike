use std::iter::Sum;
use std::ops::{Add, AddAssign};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Width {
    Bounded(usize),
    Unbounded,
}

impl Add for Width {
    type Output = Width;
    fn add(self, other: Self) -> Self::Output {
        use Width::{Bounded, Unbounded};
        match (self, other) {
            (Unbounded, _) | (_, Unbounded) => Unbounded,
            (Bounded(left), Bounded(right)) => Bounded(left + right),
        }
    }
}

impl AddAssign for Width {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sum for Width {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Width::Bounded(0), |a, b| a + b)
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn add_bounded() {
        let actual = Width::Bounded(4) + Width::Bounded(6);
        let expected = Width::Bounded(10);
        assert_eq!(expected, actual);
    }
    #[test]
    fn add_bound_unbound() {
        let actual = Width::Bounded(4) + Width::Unbounded;
        let expected = Width::Unbounded;
        assert_eq!(expected, actual);
    }
    #[test]
    fn add_unbound_bound() {
        let actual = Width::Unbounded + Width::Bounded(4);
        let expected = Width::Unbounded;
        assert_eq!(expected, actual);
    }
    #[test]
    fn add_bound_bound() {
        let actual = Width::Unbounded + Width::Unbounded;
        let expected = Width::Unbounded;
        assert_eq!(expected, actual);
    }
    #[test]
    fn sum() {
        let v = vec![Width::Bounded(5), Width::Bounded(6), Width::Bounded(7)];
        let actual: Width = v.iter().cloned().sum();
        let expected = Width::Bounded(18);
        assert_eq!(expected, actual);
    }
    #[test]
    fn add_assign() {
        let mut actual = Width::Bounded(1);
        actual += Width::Bounded(4);
        let expected = Width::Bounded(5);
        assert_eq!(expected, actual);
    }
}
