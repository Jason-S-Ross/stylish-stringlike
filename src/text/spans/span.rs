use crate::text::{
    FiniteText, Graphemes, HasWidth, Replaceable, Sliceable, Splitable, StyledGrapheme, Text, Width,
};
use ansi_term::{ANSIString, Style};
use regex::{Regex, Replacer};
use std::borrow::Cow;
use std::fmt;
use std::slice::SliceIndex;
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
impl<'a, R> Sliceable<'a, R, str> for Span<'a> {
    type Output = Span<'a>;
    type Error = ();
    fn slice(&'a self, range: R) -> Result<Self::Output, Self::Error>
    where
        R: SliceIndex<str, Output = str>,
    {
        Ok(Span {
            style: self.style.clone(),
            content: Cow::Borrowed(&self.content[range]),
        })
    }
}
impl<'a> Splitable<'a, &'a str> for Span<'a> {
    type Delim = Self;
    type Result = Self;
    #[allow(clippy::type_complexity)]
    fn split(
        &'a self,
        pattern: &'a str,
    ) -> Box<dyn Iterator<Item = (Option<Self::Result>, Option<Self::Delim>)> + 'a> {
        use std::iter::once;
        Box::new(
            (&self.content)
                .match_indices(pattern)
                // This is a silly hack to flag when we get to the last element in the list.
                // Items in the list will be Some.
                .map(Some)
                // The last item will be None.
                .chain(once(None))
                .scan(0, move |last_end, item| {
                    if let Some((start, pat)) = item {
                        let end = start + pat.len();
                        let delim = self.slice(start..end).expect("Failed to slice span");
                        let res = if start == 0 {
                            // String starts with delimiter
                            Some((None, Some(delim)))
                        } else {
                            Some((
                                Some(self.slice(*last_end..start).expect("Failed to slice span")),
                                Some(delim),
                            ))
                        };
                        *last_end = end;
                        res
                    } else {
                        // This is the last item.
                        if *last_end == self.content.len() {
                            // After consuming the last match, we are at the end of the string
                            None
                        } else {
                            // After consuming the last match, we still have some string yet
                            Some((
                                Some(
                                    self.slice(*last_end..)
                                        .expect("Failed to slice last element"),
                                ),
                                None,
                            ))
                        }
                    }
                }),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::Sliceable;
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
