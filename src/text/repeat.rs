use crate::text::{BoundedWidth, HasWidth, Joinable, Width, WidthSliceable};
use std::ops::{Bound, RangeBounds};

use std::marker::PhantomData;

#[derive(Debug)]
pub(crate) struct Repeat<'a, T> {
    content: T,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T> Repeat<'a, T> {
    pub(crate) fn new(content: T) -> Repeat<'a, T> {
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

impl<'a, T, U> WidthSliceable<'a> for Repeat<'a, T>
where
    T: BoundedWidth + WidthSliceable<'a, Output = T> + Joinable<T, Output = U>,
    U: Default + Joinable<U, Output = U> + Joinable<T, Output = U> + BoundedWidth + 'a,
{
    type Output = U;
    fn slice_width<R>(&'a self, range: R) -> Option<Self::Output>
    where
        R: RangeBounds<usize>,
    {
        let self_width = self.content.bounded_width();
        let (start, end) = match (range.start_bound(), range.end_bound()) {
            (Bound::Excluded(s), Bound::Excluded(e)) => (s.saturating_sub(1), *e),
            (Bound::Excluded(s), Bound::Included(e)) => (s.saturating_sub(1), *e + 1),
            (Bound::Included(s), Bound::Excluded(e)) => (*s, *e),
            (Bound::Included(s), Bound::Included(e)) => (*s, *e + 1),
            (Bound::Unbounded, Bound::Excluded(e)) => (0, *e),
            (Bound::Unbounded, Bound::Included(e)) => (0, *e + 1),
            _ => return None,
        };
        let (norm_start, norm_end) = {
            let shift = (start / self_width) * self_width;
            (start - shift, end - shift)
        };
        let self_range = 0..self.content.bounded_width();
        if self_range.contains(&norm_start) && self_range.contains(&norm_end) {
            // This range "fits" inside of ourselves so it's fine
            let mut res: U = Default::default();
            if let Some(s) = self.content.slice_width(start..end) {
                res = res.join(&s);
                return Some(res);
            } else {
                return None;
            }
        }
        let num_repeats =
            ((end.saturating_sub(1) / self_width) - (start / self_width)).saturating_sub(1);
        let mut res: U = Default::default();
        if let Some(s) = self.content.slice_width(norm_start..) {
            res = res.join(&s);
        }
        for _ in 0..num_repeats {
            res = res.join(&self.content);
        }
        if let Some(s) = self.content.slice_width(..norm_end % self_width) {
            res = res.join(&s);
        }
        Some(res)
    }
}

#[cfg(test)]
mod test {
    use crate::text::*;
    use ansi_term::{Color, Style};
    use std::borrow::Cow;
    use std::ops::Bound;
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
