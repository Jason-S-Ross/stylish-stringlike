use super::Painter;
use ansi_term::{ANSIString, Style};
use std::borrow::Cow;
use std::fmt;
use std::ops::Deref;

#[derive(Debug)]
pub(crate) struct Span<'a, T: Clone> {
    style: Cow<'a, T>,
    content: Cow<'a, str>,
}

impl<'a, T: Clone> Span<'a, T> {
    pub(crate) fn style(&self) -> &Cow<'a, T> {
        &self.style
    }
    pub(crate) fn content(&self) -> &Cow<'a, str> {
        &self.content
    }
    pub(crate) fn new(style: Cow<'a, T>, content: Cow<'a, str>) -> Span<'a, T> {
        Span { style, content }
    }
    pub(crate) fn borrowed(style: &'a T, content: &'a str) -> Span<'a, T> {
        Span {
            style: Cow::Borrowed(style),
            content: Cow::Borrowed(content),
        }
    }
}
impl<'a, T: Painter + Clone> fmt::Display for Span<'a, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.style.paint(self.content.as_ref()).fmt(fmt)
    }
}
impl<'a> From<&Span<'a, Style>> for ANSIString<'a> {
    fn from(span: &Span<'a, Style>) -> ANSIString<'a> {
        span.style.paint(span.content.clone())
    }
}
impl<'a> From<Span<'a, Style>> for ANSIString<'a> {
    fn from(span: Span<'a, Style>) -> ANSIString<'a> {
        span.style.paint(span.content)
    }
}
impl<'a> From<&'a ANSIString<'a>> for Span<'a, Style> {
    fn from(string: &'a ANSIString<'a>) -> Self {
        let style = Cow::Borrowed(string.style_ref());
        let content = Cow::Borrowed(string.deref());
        Span::new(style, content)
    }
}
impl<'a> From<ANSIString<'_>> for Span<'a, Style> {
    fn from(string: ANSIString<'_>) -> Self {
        let style = Cow::Owned(string.style_ref().clone());
        let content = Cow::Owned(string.deref().to_string());
        Span::new(style, content)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::{Sliceable, Split, Splitable, Width};
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
}
