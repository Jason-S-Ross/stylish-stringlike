use std::iter::Sum;
use std::ops::{Add, AddAssign};

/// An enum representing the unicode width of a (possibly infinte) text object
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Width {
    /// A finite width
    Bounded(usize),
    /// An infinite width
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

/// Support for returning the unicode width of a text object
pub trait HasWidth {
    /// Return the unicode width of an object
    ///
    /// # Example
    /// ```
    /// use stylish_stringlike::text::{HasWidth, Width};
    /// let foo = "foobar";
    /// assert_eq!(foo.width(), Width::Bounded(6));
    /// let bar = String::from("🙈🙉🙊");
    /// assert_eq!(bar.width(), Width::Bounded(6));
    /// ```
    fn width(&self) -> Width;
}

impl<T> HasWidth for T
where
    T: BoundedWidth,
{
    fn width(&self) -> Width {
        Width::Bounded(self.bounded_width())
    }
}

/// Support for returing the unicode width of text objects that are finite
pub trait BoundedWidth {
    /// Return the finite unicode width of an object
    ///
    /// # Example
    /// ```
    /// use stylish_stringlike::text::BoundedWidth;
    /// let foo = "foobar";
    /// assert_eq!(foo.bounded_width(), 6);
    /// let bar = String::from("🙈🙉🙊");
    /// assert_eq!(bar.bounded_width(), 6);
    /// ```
    fn bounded_width(&self) -> usize;
}

impl BoundedWidth for String {
    fn bounded_width(&self) -> usize {
        unicode_width::UnicodeWidthStr::width(self.as_str())
    }
}

impl BoundedWidth for &str {
    fn bounded_width(&self) -> usize {
        unicode_width::UnicodeWidthStr::width(*self)
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
