use crate::text::{
    slice_string, FiniteText, Graphemes, HasWidth, RawText, Replaceable, Sliceable, StyledGrapheme,
    Text, Width,
};
use ansi_term::{ANSIString, Style};
use regex::{Regex, Replacer};
use std::borrow::Cow;
use std::fmt;
use std::ops::RangeBounds;
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
impl<'a> Text<'a> for Span<'a> {}
impl<'a> RawText for Span<'a> {
    fn raw(&self) -> String {
        self.content.to_owned().to_string()
    }
    fn raw_ref<'b>(&self) -> &str {
        &self.content
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
impl<'a> Replaceable<&str> for Span<'a> {
    type Output = Span<'a>;
    fn replace(&self, from: &str, to: &str) -> Result<Self::Output, ()> {
        Ok(Span::owned(*self.style, self.content.replace(from, to)))
    }
    fn replace_regex<R: Replacer>(
        &self,
        searcher: &Regex,
        replacer: R,
    ) -> Result<Self::Output, ()> {
        Ok(Span::owned(
            *self.style,
            searcher.replace_all(&self.content, replacer).to_string(),
        ))
    }
}
impl<'a> Sliceable<'a> for Span<'a> {
    type Output = Span<'a>;
    type Index = usize;
    fn slice<R>(&'a self, range: R) -> Option<Self::Output>
    where
        R: RangeBounds<Self::Index> + Clone,
    {
        let s = slice_string(&self.content, range);
        if let Some(s) = s {
            Some(Span {
                style: self.style.clone(),
                content: Cow::Borrowed(s),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::{Sliceable, Splitable};
    use ansi_term::Color;
    use std::cmp::{Eq, PartialEq};

    impl<'a> Eq for Span<'a> {}
    impl<'a> PartialEq for Span<'a> {
        fn eq(&self, other: &Self) -> bool {
            &self.style == &other.style && &self.content == &other.content
        }
    }

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
        let style = Color::Blue.bold();
        let s = "foo";
        let span = Span::borrowed(&style, s);
        let expected = Width::Bounded(3);
        let actual = span.width();
        assert_eq!(expected, actual);
    }
    #[test]
    fn raw() {
        let style = Color::Blue.bold();
        let s = "foo";
        let span = Span::borrowed(&style, s);
        let expected = String::from(s);
        let actual = span.raw();
        assert_eq!(expected, actual);
    }
    #[test]
    fn replace() {
        let span = Span::owned(Color::Blue.bold(), String::from("foo"));
        let actual = span.replace("foo", "bar").unwrap();
        let expected = Span::owned(Color::Blue.bold(), String::from("bar"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn replace_regex() {
        let span = Span::owned(Color::Blue.bold(), String::from("fooooo"));
        let regex = Regex::new("fo+").unwrap();
        let actual = span.replace_regex(&regex, "bar").unwrap();
        let expected = Span::owned(Color::Blue.bold(), String::from("bar"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice() {
        let span = Span::owned(Color::Blue.bold(), String::from("0123456789"));
        let expected = Span::owned(Color::Blue.bold(), String::from("0123"));
        let actual = span.slice(0..4).unwrap();
        assert_eq!(expected, actual);
    }
    #[test]
    fn split_inner() {
        let span = Span::owned(Color::Blue.bold(), String::from("Some::Random::Path"));
        let actual = span.split("::").collect::<Vec<_>>();
        let expected = vec![
            (
                Some(Span::owned(Color::Blue.bold(), String::from("Some"))),
                Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            ),
            (
                Some(Span::owned(Color::Blue.bold(), String::from("Random"))),
                Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            ),
            (
                Some(Span::owned(Color::Blue.bold(), String::from("Path"))),
                None,
            ),
        ];
        assert_eq!(expected, actual);
    }
    #[test]
    fn split_outer() {
        let span = Span::owned(Color::Blue.bold(), String::from("::Some::Random::Path::"));
        let actual = span.split("::").collect::<Vec<_>>();
        let expected = vec![
            (
                None,
                Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            ),
            (
                Some(Span::owned(Color::Blue.bold(), String::from("Some"))),
                Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            ),
            (
                Some(Span::owned(Color::Blue.bold(), String::from("Random"))),
                Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            ),
            (
                Some(Span::owned(Color::Blue.bold(), String::from("Path"))),
                Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            ),
        ];
        assert_eq!(expected, actual);
    }
    #[test]
    fn split_left() {
        let span = Span::owned(Color::Blue.bold(), String::from("::Some::Random::Path"));
        let actual = span.split("::").collect::<Vec<_>>();
        let expected = vec![
            (
                None,
                Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            ),
            (
                Some(Span::owned(Color::Blue.bold(), String::from("Some"))),
                Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            ),
            (
                Some(Span::owned(Color::Blue.bold(), String::from("Random"))),
                Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            ),
            (
                Some(Span::owned(Color::Blue.bold(), String::from("Path"))),
                None,
            ),
        ];
        assert_eq!(expected, actual);
    }
    #[test]
    fn split_right() {
        let span = Span::owned(Color::Blue.bold(), String::from("Some::Random::Path::"));
        let actual = span.split("::").collect::<Vec<_>>();
        let expected = vec![
            (
                Some(Span::owned(Color::Blue.bold(), String::from("Some"))),
                Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            ),
            (
                Some(Span::owned(Color::Blue.bold(), String::from("Random"))),
                Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            ),
            (
                Some(Span::owned(Color::Blue.bold(), String::from("Path"))),
                Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            ),
        ];
        assert_eq!(expected, actual);
    }
}
