use crate::text::{BoundedWidth, HasWidth, Joinable, Width, WidthSliceable};
use std::ops::{Bound, RangeBounds};

use std::marker::PhantomData;

/// A text widget that repeats its content arbitrarily many times.
#[derive(Debug)]
pub struct Repeat<'a, T> {
    content: T,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T> Repeat<'a, T> {
    pub fn new(content: T) -> Repeat<'a, T> {
        Repeat {
            content,
            _marker: Default::default(),
        }
    }
}

impl<'a, T> HasWidth for Repeat<'a, T> {
    fn width(&self) -> Width {
        Width::Unbounded
    }
}

impl<'a, T, U> WidthSliceable for Repeat<'a, T>
where
    T: BoundedWidth + WidthSliceable<Output = T> + Joinable<T, Output = U>,
    U: Default + Joinable<U, Output = U> + Joinable<T, Output = U> + BoundedWidth + 'a,
{
    type Output = U;
    fn slice_width<R>(&self, range: R) -> Option<Self::Output>
    where
        R: RangeBounds<usize>,
    {
        use std::ops::Bound::*;
        fn shift_range<R: RangeBounds<usize>>(
            range: &R,
            shift: i32,
        ) -> Option<(Bound<usize>, Bound<usize>)> {
            fn ss(target: usize, shift: i32) -> usize {
                if shift < 0 {
                    target.saturating_sub(shift.abs() as usize)
                } else {
                    target + shift as usize
                }
            }
            let start = match range.start_bound() {
                Excluded(s) => Excluded(ss(*s, shift)),
                Included(s) => Included(ss(*s, shift)),
                Unbounded => Unbounded,
            };
            let end = match range.end_bound() {
                Excluded(e) => Excluded(ss(*e, shift)),
                Included(e) => {
                    if *e as i32 + shift < 0 {
                        Excluded(0)
                    } else {
                        Included(ss(*e, shift))
                    }
                }
                Unbounded => return None,
            };
            Some((start, end))
        }
        let target_width = match (range.start_bound(), range.end_bound()) {
            (_, Unbounded) => return None,
            (Unbounded, Excluded(e)) => *e,
            (Unbounded, Included(e)) => *e + 1,
            (Included(s), Excluded(e)) => e.saturating_sub(*s),
            (Included(s), Included(e)) => (*e + 1).saturating_sub(*s),
            (Excluded(s), Excluded(e)) => e.saturating_sub(*s + 1),
            (Excluded(s), Included(e)) => (*e + 1).saturating_sub(*s + 1),
        };
        if target_width == 0 {
            return None;
        }
        let self_width = self.content.bounded_width();

        if self_width == 0 {
            return None;
        }
        let mut res: U = Default::default();

        let mut segment = 0;
        let mut started = false;
        loop {
            let shifted_range = shift_range(&range, -((segment * self_width) as i32));
            if let Some(shifted_range) = shifted_range {
                let sliced = self.content.slice_width(shifted_range);
                if let Some(sliced) = sliced {
                    started = true;
                    res = res.join(&sliced)
                } else if started {
                    break;
                }
            } else {
                return None;
            }
            segment += 1;
            if segment > 10 {
                return Some(res);
            }
        }

        Some(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::*;
    use ansi_term::{Color, Style};
    use std::borrow::Cow;
    #[test]
    fn make_repeat_trivial_null() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("")),
        );
        let repeat = Repeat::new(span);
        let actual = repeat.slice_width(1..100);
        let expected = None;
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_trivial_empty() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("0")),
        );
        let repeat = Repeat::new(span);
        let actual = repeat.slice_width(0..0);
        let expected = None;
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_trivial_inclusive() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("0")),
        );
        let repeat = Repeat::new(span);
        let res = repeat.slice_width(..=0);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Yellow.paint("0"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_trivial_unbounded() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("0")),
        );
        let repeat = Repeat::new(span);
        let actual = repeat.slice_width(0..);
        let expected = None;
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_trivial_single() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("0")),
        );
        let repeat = Repeat::new(span);
        let res = repeat.slice_width(..1);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Yellow.paint("0"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_trivial_multiple() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("0")),
        );
        let repeat = Repeat::new(span);
        let res = repeat.slice_width(..2);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Yellow.paint("00"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_long() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("01234")),
        );
        let repeat = Repeat::new(span);
        let res = repeat.slice_width(1..14);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Yellow.paint("1234012340123"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_short() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("01234")),
        );
        let repeat = Repeat::new(span);
        let res = repeat.slice_width(..3);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Yellow.paint("012"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_mid_inclusive() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("01234")),
        );
        let repeat = Repeat::new(span);
        let res = repeat.slice_width(1..=3);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Yellow.paint("123"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_mid_ex_in() {
        use std::ops::Bound::*;
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("01234")),
        );
        let repeat = Repeat::new(span);
        let res = repeat.slice_width((Excluded(1), Included(3)));
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Yellow.paint("23"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_mid_ex_ex() {
        use std::ops::Bound::*;
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("01234")),
        );
        let repeat = Repeat::new(span);
        let res = repeat.slice_width((Excluded(1), Excluded(3)));
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Yellow.paint("2"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_left() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("01234")),
        );
        let repeat = Repeat::new(span);
        let res = repeat.slice_width(1..9);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Yellow.paint("12340123"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_shifted_long() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("01234")),
        );
        let repeat = Repeat::new(span);
        let res = repeat.slice_width(7..18);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Yellow.paint("23401234012"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_repeat_shifted_extra_long() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Yellow.normal()),
            Cow::Owned(String::from("01234")),
        );
        let repeat = Repeat::new(span);
        let res = repeat.slice_width(7..23);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Yellow.paint("2340123401234012"));
        assert_eq!(expected, actual);
    }
}
