use crate::text::{
    slice_string, FiniteText, Graphemes, HasWidth, RawText, Replaceable, Sliceable, StyledGrapheme,
    Text, Width,
};
use ansi_term::{ANSIString, Style};
use regex::{Regex, Replacer};
use std::borrow::{Borrow, Cow};
use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::ops::RangeBounds;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct Span<'a, T: Clone> {
    style: Cow<'a, T>,
    content: Cow<'a, str>,
}

impl<'a, T: Clone> Span<'a, T> {
    pub fn style(&self) -> &Cow<'a, T> {
        &self.style
    }
    pub fn content(&self) -> &Cow<'a, str> {
        &self.content
    }
    pub fn borrowed(style: &'a T, content: &'a str) -> Span<'a, T> {
        Span {
            style: Cow::Borrowed(style),
            content: Cow::Borrowed(content),
        }
    }
    pub fn owned(style: T, content: String) -> Span<'a, T> {
        Span {
            style: Cow::Owned(style),
            content: Cow::Owned(content),
        }
    }
}
impl<'a> fmt::Display for Span<'a, Style> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.style.paint(self.content.as_ref()).fmt(fmt)
    }
}
impl<'a, T> Graphemes<'a, T> for Span<'a, T>
where
    T: Clone + 'a,
{
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a, T>> + 'a> {
        Box::new(
            self.content
                .graphemes(true)
                .map(move |grapheme| StyledGrapheme::borrowed(self.style.borrow(), grapheme)),
        )
    }
}
impl<'a, T> HasWidth for Span<'a, T>
where
    T: Clone,
{
    fn width(&self) -> Width {
        self.graphemes().map(|x| x.width()).sum()
    }
}
impl<'a, T: Clone> Text<'a, T> for Span<'a, T> {}
impl<'a, T: Clone> RawText for Span<'a, T> {
    fn raw(&self) -> String {
        self.content.to_owned().to_string()
    }
    fn raw_ref<'b>(&self) -> &str {
        &self.content
    }
}
impl<'a, T: Clone> FiniteText<'a, T> for Span<'a, T> {}
impl<'a> From<&'a Span<'a, Style>> for ANSIString<'a> {
    fn from(span: &'a Span<'a, Style>) -> ANSIString<'a> {
        span.style.paint(span.content.as_ref())
    }
}
impl<'a> From<Span<'a, Style>> for ANSIString<'a> {
    fn from(span: Span<'a, Style>) -> ANSIString<'a> {
        span.style.paint(span.content)
    }
}

impl<'a, T> Replaceable<'a, &'a str> for Span<'a, T>
where
    T: Clone,
{
    fn replace(&'a self, from: &str, replacer: &'a str) -> Self {
        let mut content = String::new();
        let mut last_end = 0;
        for (start, part) in self.content.match_indices(from) {
            if let Some(s) = self.slice(last_end..start) {
                content += &s.content;
                content.push_str(&replacer);
            }
            last_end = start + part.len();
        }
        if let Some(s) = self.content.get(last_end..) {
            content += s;
        }
        Span {
            style: self.style.clone(),
            content: Cow::Owned(content),
        }
    }
    fn replace_regex(&'a self, searcher: &Regex, replacer: &'a str) -> Self {
        let captures = searcher.captures_iter(&self.content);
        let mut last_end = 0;
        let mut content = String::new();
        for capture in captures {
            let mat = capture
                .get(0)
                .expect("Captures are always supposed to have one match");
            if let Some(s) = self.slice(last_end..mat.start()) {
                content += &s.content;
                let mut new = String::new();
                String::from(replacer).replace_append(&capture, &mut new);
                content += &new;
            }
            last_end = mat.end();
        }
        if let Some(s) = self.content.get(last_end..) {
            content += s;
        }
        Span {
            style: self.style.clone(),
            content: Cow::Owned(content),
        }
    }
}
impl<'a, T: Clone> Sliceable<'a> for Span<'a, T> {
    type Output = Span<'a, T>;
    type Index = usize;
    fn slice<R>(&'a self, range: R) -> Option<Self::Output>
    where
        R: RangeBounds<Self::Index> + Clone,
    {
        let s = slice_string(&self.content, range);
        if let Some(s) = s {
            Some(Span::borrowed(&self.style, s))
        } else {
            None
        }
    }
}

impl<'a, T: Clone + PartialEq> Eq for Span<'a, T> {}
impl<'a, T: Clone + PartialEq> std::cmp::PartialEq for Span<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.style == other.style && self.content == other.content
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::{Sliceable, Split, Splitable};
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
        let actual = span.replace("foo", "bar");
        let expected = Span::owned(Color::Blue.bold(), String::from("bar"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn replace_regex() {
        let span = Span::owned(Color::Blue.bold(), String::from("fooooo"));
        let regex = Regex::new("fo+").unwrap();
        let actual = span.replace_regex(&regex, "bar");
        let expected = Span::owned(Color::Blue.bold(), String::from("bar"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn replace_regex_captures() {
        let span = Span::owned(Color::Blue.bold(), String::from("One Two Three"));
        let regex = Regex::new(r"(\w+) (\w+) (\w+)").unwrap();
        let actual = span.replace_regex(&regex, "$3 $2 $1");
        let expected = Span::owned(Color::Blue.bold(), String::from("Three Two One"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn replace_regex_captures_multiple() {
        let span = Span::owned(
            Color::Blue.bold(),
            String::from("123-456-7890  132/465/0987  111_222_3333"),
        );
        let regex = Regex::new(r"([0-9]{3})[_/-]([0-9]{3})[_/-]([0-9]{4})").unwrap();
        let actual = span.replace_regex(&regex, "$1-$2-$3");
        let expected = Span::owned(
            Color::Blue.bold(),
            String::from("123-456-7890  132-465-0987  111-222-3333"),
        );
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
        let actual = span.split_style("::").collect::<Vec<_>>();
        let expected = vec![
            Split {
                segment: Some(Span::owned(Color::Blue.bold(), String::from("Some"))),
                delim: Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            },
            Split {
                segment: Some(Span::owned(Color::Blue.bold(), String::from("Random"))),
                delim: Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            },
            Split {
                segment: Some(Span::owned(Color::Blue.bold(), String::from("Path"))),
                delim: None,
            },
        ];
        assert_eq!(expected, actual);
    }
    #[test]
    fn split_outer() {
        let span = Span::owned(Color::Blue.bold(), String::from("::Some::Random::Path::"));
        let actual = span.split_style("::").collect::<Vec<_>>();
        let expected = vec![
            Split {
                segment: None,
                delim: Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            },
            Split {
                segment: Some(Span::owned(Color::Blue.bold(), String::from("Some"))),
                delim: Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            },
            Split {
                segment: Some(Span::owned(Color::Blue.bold(), String::from("Random"))),
                delim: Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            },
            Split {
                segment: Some(Span::owned(Color::Blue.bold(), String::from("Path"))),
                delim: Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            },
        ];
        assert_eq!(expected, actual);
    }
    #[test]
    fn split_left() {
        let span = Span::owned(Color::Blue.bold(), String::from("::Some::Random::Path"));
        let actual = span.split_style("::").collect::<Vec<_>>();
        let expected = vec![
            Split {
                segment: None,
                delim: Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            },
            Split {
                segment: Some(Span::owned(Color::Blue.bold(), String::from("Some"))),
                delim: Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            },
            Split {
                segment: Some(Span::owned(Color::Blue.bold(), String::from("Random"))),
                delim: Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            },
            Split {
                segment: Some(Span::owned(Color::Blue.bold(), String::from("Path"))),
                delim: None,
            },
        ];
        assert_eq!(expected, actual);
    }
    #[test]
    fn split_right() {
        let span = Span::owned(Color::Blue.bold(), String::from("Some::Random::Path::"));
        let actual = span.split_style("::").collect::<Vec<_>>();
        let expected = vec![
            Split {
                segment: Some(Span::owned(Color::Blue.bold(), String::from("Some"))),
                delim: Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            },
            Split {
                segment: Some(Span::owned(Color::Blue.bold(), String::from("Random"))),
                delim: Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            },
            Split {
                segment: Some(Span::owned(Color::Blue.bold(), String::from("Path"))),
                delim: Some(Span::owned(Color::Blue.bold(), String::from("::"))),
            },
        ];
        assert_eq!(expected, actual);
    }
}
