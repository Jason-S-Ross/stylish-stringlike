use super::{
    slice_string, BoundedWidth, Expandable, Joinable, Paintable, Pushable, RawText, Sliceable,
    Spans,
};
#[cfg(test)]
use ansi_term::{ANSIString, Style};
use regex::Captures;
use std::borrow::Cow;
use std::fmt;
#[cfg(test)]
use std::ops::Deref;
use std::ops::RangeBounds;
use unicode_width::UnicodeWidthStr;

/// A span of text having a single style.
#[derive(Debug, Default, PartialEq)]
pub struct Span<'a, T: Clone> {
    style: Cow<'a, T>,
    content: Cow<'a, str>,
}

impl<'a, T: Clone> Span<'a, T> {
    pub fn style(&self) -> &Cow<'a, T> {
        &self.style
    }
    pub fn new(style: Cow<'a, T>, content: Cow<'a, str>) -> Span<'a, T> {
        Span { style, content }
    }
    pub fn borrowed(style: &'a T, content: &'a str) -> Span<'a, T> {
        Span {
            style: Cow::Borrowed(style),
            content: Cow::Borrowed(content),
        }
    }
}
impl<'a, T: Paintable + Clone> fmt::Display for Span<'a, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.style.paint(self.content.as_ref()).fmt(fmt)
    }
}

#[cfg(test)]
impl<'a> From<&Span<'a, Style>> for ANSIString<'a> {
    fn from(span: &Span<'a, Style>) -> ANSIString<'a> {
        span.style.paint(span.content.clone())
    }
}
#[cfg(test)]
impl<'a> From<Span<'a, Style>> for ANSIString<'a> {
    fn from(span: Span<'a, Style>) -> ANSIString<'a> {
        span.style.paint(span.content)
    }
}
#[cfg(test)]
impl<'a> From<&'a ANSIString<'a>> for Span<'a, Style> {
    fn from(string: &'a ANSIString<'a>) -> Self {
        let style = Cow::Borrowed(string.style_ref());
        let content = Cow::Borrowed(string.deref());
        Span::new(style, content)
    }
}
#[cfg(test)]
impl<'a> From<ANSIString<'_>> for Span<'a, Style> {
    fn from(string: ANSIString<'_>) -> Self {
        let style = Cow::Owned(*string.style_ref());
        let content = Cow::Owned(string.deref().to_string());
        Span::new(style, content)
    }
}

impl<'a, T: Clone + Default + PartialEq> Joinable<Span<'a, T>> for Span<'a, T> {
    type Output = Spans<T>;
    fn join(&self, other: &Span<T>) -> Self::Output {
        let mut res: Spans<T> = Default::default();
        res.push(self);
        res.push(other);
        res
    }
}
impl<'a, T: Clone> Pushable<str> for Span<'a, T> {
    fn push(&mut self, other: &str) {
        self.content.to_mut().push_str(other);
    }
}
impl<'a, T: Clone> Sliceable<'a> for Span<'a, T> {
    fn slice<R>(&'a self, range: R) -> Option<Self>
    where
        R: RangeBounds<usize> + Clone,
    {
        let string = slice_string(&self.content, range);
        string.map(|string| Span::new(self.style.clone(), Cow::Borrowed(string)))
    }
}
impl<'a, T: Clone> RawText for Span<'a, T> {
    fn raw(&self) -> String {
        self.content.to_string()
    }
    fn raw_ref(&self) -> &str {
        &self.content
    }
}
impl<'a, T: Clone> BoundedWidth for Span<'a, T> {
    fn bounded_width(&self) -> usize {
        self.content.width()
    }
}
impl<'a, T: Clone> Expandable for Span<'a, T> {
    fn expand(&self, capture: &Captures) -> Span<'a, T> {
        let new_content = self.raw().expand(capture);
        Span {
            style: self.style.clone(),
            content: Cow::Owned(new_content),
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::text::{Sliceable, WidthSliceable};
    use ansi_term::Color;

    #[test]
    fn convert() {
        let style = Style::new();
        let span = Span::borrowed(&style, "foo");
        let actual: ANSIString = (&span).into();
        let expected = Style::new().paint("foo");
        assert_eq!(expected, actual);
    }
    #[test]
    fn fmt() {
        let style = Style::new();
        let span = Span::borrowed(&style, "foo");
        let foo: ANSIString = (&span).into();
        let actual = format!("{}", span);
        let expected = format!("{}", foo);
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("012345678")),
        );
        let res = span.slice(1..8);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Black.paint("1234567"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_middle() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("012345678")),
        );
        let res = span.slice_width(1..2);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Black.paint("1"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_left() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("012345678")),
        );
        let res = span.slice_width(..1);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Black.paint("0"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_right() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("012345678")),
        );
        let res = span.slice_width(8..);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Black.paint("8"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_emoji_left_none() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("😼🙋👩📪")),
        );
        let actual = span.slice_width(..1);
        let expected = None;
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_emoji_left_some() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("😼🙋👩📪")),
        );
        let res = span.slice_width(..2);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Black.paint("😼"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_emoji_left_more() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("😼🙋👩📪")),
        );
        let res = span.slice_width(..3);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Black.paint("😼"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_emoji_left_even_more() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("😼🙋👩📪")),
        );
        let res = span.slice_width(..4);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Black.paint("😼🙋"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_emoji_middle_none_less() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("😼🙋👩📪")),
        );
        let res = span.slice_width(1..2);
        let actual = res;
        let expected = None;
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_emoji_middle_none_more() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("😼🙋👩📪")),
        );
        let res = span.slice_width(1..3);
        let actual = res;
        let expected = None;
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_emoji_middle_some() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("😼🙋👩📪")),
        );
        let res = span.slice_width(1..4);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Black.paint("🙋"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_emoji_middle_more() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("😼🙋👩📪")),
        );
        let res = span.slice_width(1..6);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Black.paint("🙋👩"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_emoji_right_none_trivial() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("😼🙋👩📪")),
        );
        let actual = span.slice_width(8..);
        let expected = None;
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_emoji_right_none_simple() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("😼🙋👩📪")),
        );
        let actual = span.slice_width(7..);
        let expected = None;
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_emoji_right_some() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("😼🙋👩📪")),
        );
        let res = span.slice_width(6..);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Black.paint("📪"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_width_full() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Black.normal()),
            Cow::Owned(String::from("😼🙋👩📪")),
        );
        let res = span.slice_width(..);
        let actual = format!("{}", res.unwrap());
        let expected = format!("{}", Color::Black.paint("😼🙋👩📪"));
        assert_eq!(expected, actual);
    }
}
