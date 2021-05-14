use crate::text::{FiniteText, Graphemes, HasWidth, StyledGrapheme, Text, Width};
use ansi_term::{ANSIString, Style};
use std::borrow::Cow;
use std::fmt;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct Span<'a> {
    style: Cow<'a, Style>,
    content: Cow<'a, str>,
}
impl<'a> fmt::Display for Span<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.style.paint(self.content.as_ref()).fmt(fmt)
    }
}
impl<'a> Graphemes<'a> for Span<'a> {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        Box::new(
            self.content
                .graphemes(true)
                .map(move |grapheme| StyledGrapheme::borrowed(&self.style, grapheme)),
        )
    }
}
impl<'a> HasWidth for Span<'a> {
    fn width(&self) -> Width {
        self.graphemes().map(|x| x.width()).sum()
    }
}
impl<'a> Text<'a> for Span<'a> {
    fn raw(&self) -> String {
        self.content.to_owned().to_string()
    }
}
impl<'a> FiniteText<'a> for Span<'a> {}
impl<'a> Span<'a> {
    pub fn borrowed(style: &'a Style, content: &'a str) -> Span<'a> {
        Span {
            style: Cow::Borrowed(style),
            content: Cow::Borrowed(content),
        }
    }
    pub fn owned(style: Style, content: String) -> Span<'a> {
        Span {
            style: Cow::Owned(style),
            content: Cow::Owned(content),
        }
    }
}
impl<'a> From<&'a Span<'a>> for ANSIString<'a> {
    fn from(span: &'a Span<'a>) -> ANSIString<'a> {
        span.style.paint(span.content.as_ref())
    }
}
impl<'a> From<Span<'a>> for ANSIString<'a> {
    fn from(span: Span<'a>) -> ANSIString<'a> {
        span.style.paint(span.content)
    }
}

#[cfg(test)]
mod test {
    use super::*;
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
    fn graphemes() {
        use ansi_term::Color;
        let style = Color::Blue.bold();
        let s = "foo";
        let span = Span::borrowed(&style, s);
        let c = &s[..1];
        let expected = StyledGrapheme::borrowed(&style, c);
        let actual = span.graphemes().next().unwrap();
        assert_eq!(expected, actual);
    }
    #[test]
    fn width() {
        use ansi_term::Color;
        let style = Color::Blue.bold();
        let s = "foo";
        let span = Span::borrowed(&style, s);
        let expected = Width::Bounded(3);
        let actual = span.width();
        assert_eq!(expected, actual);
    }
    #[test]
    fn raw() {
        use ansi_term::Color;
        let style = Color::Blue.bold();
        let s = "foo";
        let span = Span::borrowed(&style, s);
        let expected = String::from(s);
        let actual = span.raw();
        assert_eq!(expected, actual);
    }
}
