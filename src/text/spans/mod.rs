mod search_tree;
mod span;
use super::{
    slice_string, BoundedWidth, Expandable, HasWidth, Joinable, Paintable, Pushable, RawText,
    Replaceable, Sliceable, Width,
};

use regex::{Captures, Regex, Replacer};
use search_tree::SearchTree;
pub use span::Span;
use std::borrow::Borrow;
use std::borrow::Cow;
use std::fmt;
use std::iter::FromIterator;
use std::iter::{once, repeat};
use std::ops::RangeBounds;
/// A string with various styles applied to the span.
/// Styles do not not cascade. Only the most recent style
/// applies to the current character.
#[derive(Clone, Debug)]
pub struct Spans<T> {
    content: String,
    /// Byte-indexed map of spans
    spans: SearchTree<T>,
}

impl<T> Default for Spans<T> {
    fn default() -> Self {
        Self {
            content: String::new(),
            spans: Default::default(),
        }
    }
}

impl<T: PartialEq> Eq for Spans<T> {}

impl<T: PartialEq> PartialEq for Spans<T> {
    fn eq(&self, other: &Spans<T>) -> bool {
        self.content == other.content && self.spans == other.spans
    }
}

impl<T> Spans<T> {
    #[allow(clippy::type_complexity)]
    fn segments(
        &self,
    ) -> Box<dyn Iterator<Item = ((&usize, Cow<'_, T>), Option<(&usize, Cow<'_, T>)>)> + '_>
    where
        T: Clone + Default,
    {
        if self.spans.contains_key(0) {
            Box::new(
                self.spans
                    .iter()
                    .map(|(key, val)| (key, Cow::Borrowed(val)))
                    .zip(
                        self.spans
                            .iter()
                            .map(|(key, val)| (key, Cow::Borrowed(val)))
                            .map(Some)
                            .skip(1)
                            .chain(repeat(None)),
                    ),
            )
        } else {
            Box::new(
                once((&0, Cow::Owned(Default::default())))
                    .chain(
                        self.spans
                            .iter()
                            .map(|(key, val)| (key, Cow::Borrowed(val))),
                    )
                    .zip(
                        self.spans
                            .iter()
                            .map(|(key, val)| (key, Cow::Borrowed(val)))
                            .map(Some)
                            .chain(repeat(None)),
                    ),
            )
        }
    }
    /// Returns the spans of text contained in this object.
    pub fn spans(&self) -> impl Iterator<Item = Span<'_, T>>
    where
        T: Clone + Default,
    {
        self.segments()
            .filter_map(move |((first_key, style), second)| {
                let second_key = if let Some((second_key, _)) = second {
                    *second_key
                } else {
                    self.content.len()
                };
                #[allow(clippy::manual_map)]
                if let Some(ref s) = self.content.get(*first_key..second_key) {
                    Some(Span::new(style, Cow::Borrowed(s)))
                } else {
                    // This represents an invalid state in the spans.
                    // One of the spans is actually out of the range of the length of the string.
                    None
                }
            })
    }
    fn trim(&mut self) {
        self.spans.trim(self.content.len().saturating_sub(1));
    }
}

impl<T: Clone + PartialEq> Pushable<Spans<T>> for Spans<T> {
    fn push(&mut self, other: &Spans<T>) {
        // copy_with_shift always succeeds because len is always positive so no
        // risk converting
        self.spans
            .copy_with_shift(&other.spans, .., self.content.len())
            .unwrap();
        self.content.push_str(&other.content);
        self.trim();
    }
}

impl<'a, T: Clone + PartialEq> Pushable<Span<'a, T>> for Spans<T> {
    fn push(&mut self, other: &Span<'a, T>) {
        self.spans
            .insert(self.content.len(), other.style().clone().into_owned());
        self.content.push_str(other.raw_ref());
        self.spans.dedup();
        self.trim();
    }
}

impl<T> Pushable<str> for Spans<T> {
    fn push(&mut self, other: &str) {
        self.content.push_str(other);
    }
}

impl<T: Default + Clone + PartialEq> Expandable for Spans<T> {
    fn expand(&self, capture: &Captures) -> Self {
        let mut result: Spans<T> = Default::default();
        for span in self.spans() {
            let mut dst = String::new();
            capture.expand(span.raw_ref(), &mut dst);
            let new_span = Span::<T>::borrowed(&span.style(), &dst);
            result.push(&new_span);
        }
        result
    }
}

// Did a specific impl for this because I haven't figured out how to get
// a blanket impl over string that works properly
impl<'a, T: Clone + PartialEq> Replaceable<'a, &'a str> for Spans<T> {
    fn replace(&self, from: &str, replacer: &'a str) -> Self {
        let mut result = Spans {
            content: String::new(),
            spans: SearchTree::new(),
        };

        let mut last_end = 0;
        for (start, part) in self.content.match_indices(from) {
            if let Some(spans) = self.slice(last_end..start) {
                result.push(&spans);
                if let Some(mut r) = self.slice(start..start + part.len()) {
                    r.content = String::from(replacer);
                    result.push(&r);
                }
            }
            last_end = start + part.len();
        }
        if let Some(spans) = self.slice(last_end..) {
            result.push(&spans);
        }
        result.trim();
        result
    }
    fn replace_regex(&self, searcher: &Regex, replacer: &'a str) -> Self {
        let mut last_end = 0;
        let mut result = Spans {
            content: String::new(),
            spans: SearchTree::new(),
        };
        let captures = searcher.captures_iter(&self.content);
        for capture in captures {
            let mat = capture
                .get(0)
                .expect("Captures are always supposed to have one match");
            if let Some(spans) = self.slice(last_end..mat.start()) {
                result.push(&spans);
                if let Some(mut r) = self.slice(mat.start()..mat.end()) {
                    let mut new = String::new();
                    String::from(replacer).replace_append(&capture, &mut new);
                    r.content = new;
                    result.push(&r);
                }
                last_end = mat.end();
            }
        }
        if let Some(spans) = self.slice(last_end..) {
            result.push(&spans);
        }
        result.trim();
        result
    }
}

impl<'a, T: Clone> Sliceable<'a> for Spans<T> {
    fn slice<R>(&'a self, range: R) -> Option<Self>
    where
        R: RangeBounds<usize> + Clone,
    {
        let string = slice_string(&self.content, range.clone());
        if self.spans.is_empty() {
            if let Some(string) = string {
                return Some(Spans {
                    content: string.to_string(),
                    spans: SearchTree::new(),
                });
            }
        }
        let spans = self.spans.slice(range);
        if let (Some(string), Some(spans)) = (string, spans) {
            Some(Spans {
                content: string.to_string(),
                spans,
            })
        } else {
            None
        }
    }
}

impl<'a, T, U> FromIterator<U> for Spans<T>
where
    T: Clone + PartialEq + 'a,
    U: Borrow<Spans<T>> + 'a,
{
    fn from_iter<I>(iter: I) -> Spans<T>
    where
        I: IntoIterator<Item = U>,
    {
        let mut result: Spans<T> = Default::default();
        for span in iter {
            result.push(span.borrow());
        }
        result.spans.dedup();
        result
    }
}

impl<'a, T> FromIterator<Span<'a, T>> for Spans<T>
where
    T: Clone + PartialEq + 'a,
{
    fn from_iter<I>(iter: I) -> Spans<T>
    where
        I: IntoIterator<Item = Span<'a, T>>,
    {
        let mut result: Spans<T> = Default::default();
        for span in iter {
            result.push(&span);
        }
        result.spans.dedup();
        result
    }
}

impl<T> RawText for Spans<T> {
    fn raw(&self) -> String {
        self.content.clone()
    }
    fn raw_ref<'a>(&self) -> &str {
        &self.content
    }
}

impl<T> From<&str> for Spans<T>
where
    T: Clone + Default + PartialEq,
{
    fn from(other: &str) -> Spans<T> {
        let mut spans: SearchTree<_> = Default::default();
        spans.insert(0, Default::default());
        Spans {
            content: String::from(other),
            spans,
        }
    }
}

impl<'a, T: Paintable + Clone + Default> fmt::Display for Spans<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        T::paint_many(self.spans().map(|span| (span.style().clone(), span.raw()))).fmt(fmt)
    }
}

impl<T> BoundedWidth for Spans<T> {
    fn bounded_width(&self) -> usize {
        self.content.bounded_width()
    }
}

impl<T> HasWidth for Spans<T> {
    fn width(&self) -> Width {
        Width::Bounded(self.bounded_width())
    }
}

impl<T: PartialEq + Clone> Joinable<Spans<T>> for Spans<T> {
    type Output = Spans<T>;
    fn join(&self, other: &Spans<T>) -> Self::Output {
        let mut result: Spans<T> = Default::default();
        result.push(self);
        result.push(other);
        result.trim();
        result
    }
}

impl<T: PartialEq + Clone> Joinable<Span<'_, T>> for Spans<T> {
    type Output = Spans<T>;
    fn join(&self, other: &Span<'_, T>) -> Self::Output {
        let mut result: Spans<T> = Default::default();
        result.push(self);
        result.push(other);
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::{Sliceable, Split, Splitable, WidthSliceable};
    use ansi_term::{ANSIString, ANSIStrings, Color, Style};
    fn strings_to_spans(strings: &[ANSIString<'_>]) -> Spans<Style> {
        strings.iter().map(Span::<Style>::from).collect()
    }
    fn string_to_spans(string: &ANSIString<'_>) -> Spans<Style> {
        let span = Span::<Style>::from(string);
        let mut spans: Spans<Style> = Default::default();
        spans.push(&span);
        spans
    }
    #[test]
    fn test_slice_width_easy() {
        let text = strings_to_spans(&[Color::Green.paint("foo")]);
        let actual = text.slice_width(..2).unwrap();
        let expected = strings_to_spans(&[Color::Green.paint("fo")]);
        assert_eq!(expected, actual);
    }
    #[test]
    fn test_slice_width_left_hard() {
        let text = strings_to_spans(&[Color::Green.paint("👱👱👱")]);
        let actual = text.slice_width(..3).unwrap();
        let expected = strings_to_spans(&[Color::Green.paint("👱")]);
        assert_eq!(expected, actual);
        let actual = text.slice_width(..4).unwrap();
        let expected = strings_to_spans(&[Color::Green.paint("👱👱")]);
        assert_eq!(expected, actual);
    }
    #[test]
    fn test_finite_width() {
        let text = strings_to_spans(&[Color::Green.paint("foo")]);
        let expected = 3;
        let actual = text.bounded_width();
        assert_eq!(expected, actual);
    }
    #[test]
    fn build_span() {
        let from = Color::Green.paint("foo");
        let to = string_to_spans(&from);
        let expected = format!("{}", from);
        let actual = format!("{}", to);
        assert_eq!(expected, actual);
    }
    #[test]
    fn build_spans() {
        let texts = [
            Color::Red.paint("a"),
            Color::Blue.paint("b"),
            Color::Blue.paint("⛇"),
        ];
        let text = strings_to_spans(&texts);
        let string = ANSIStrings(&texts);
        let strings = text.spans().map(ANSIString::from).collect::<Vec<_>>();
        let output = ANSIStrings(&strings);
        let expected = format!("{}", string);
        let actual = format!("{}", output);
        assert_eq!(expected, actual);
    }
    #[test]
    fn simple_replace() {
        let text = strings_to_spans(&[Color::Red.paint("foo")]);
        let actual = text.replace("foo", "bar");
        let expected = strings_to_spans(&[Color::Red.paint("bar")]);
        assert_eq!(expected, actual);
    }
    #[test]
    fn replace_in_span() {
        let text = strings_to_spans(&[Color::Red.paint("Bob "), Color::Blue.paint("Dylan")]);
        let new_text = text.replace("Bob", "Robert");
        let target_text =
            strings_to_spans(&[Color::Red.paint("Robert "), Color::Blue.paint("Dylan")]);
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_chars() {
        let text = strings_to_spans(&[
            Color::Blue.paint("what"),
            Color::Red.paint("//\\/;,!"),
            Color::Blue.paint("the fudge"),
        ]);
        let new_text = text.replace("/", "/");
        let target_text = strings_to_spans(&[
            Color::Blue.paint("what"),
            Color::Red.paint("//\\/;,!"),
            Color::Blue.paint("the fudge"),
        ]);
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_across_span_simple_2() {
        let text = strings_to_spans(&[
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("oo foo fo"),
            Color::Green.paint("o"),
        ]);
        let new_text = text.replace("foo", "bar");
        let target_text = strings_to_spans(&[
            Color::Red.paint("Here is some b"),
            Color::Blue.paint("ar bar ba"),
            Color::Green.paint("r"),
        ]);
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn simple_regex_replace() {
        let text = strings_to_spans(&[Color::Red.paint("foooo")]);
        let new_text = text.replace_regex(&Regex::new("fo+").unwrap(), "bar");
        let target_text = strings_to_spans(&[Color::Red.paint("bar")]);

        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_across_span_simple_trival() {
        let text = strings_to_spans(&[Color::Red.paint("Here lies "), Color::Blue.paint("Beavis")]);
        let new_text = text.replace_regex(
            &Regex::new(r"(Here lies) Beavis").unwrap(),
            "Here lies Butthead",
        );
        let target_text = strings_to_spans(&[
            Color::Red.paint("Here lies "),
            Color::Blue.paint("Butthead"),
        ]);
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_across_span_simple_backref() {
        let text = strings_to_spans(&[Color::Red.paint("Here lies "), Color::Blue.paint("Beavis")]);
        let new_text =
            text.replace_regex(&Regex::new(r"(Here lies) Beavis").unwrap(), "$1 Butthead");
        let target_text = strings_to_spans(&[
            Color::Red.paint("Here lies "),
            Color::Blue.paint("Butthead"),
        ]);
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_across_span_simple_2_backref() {
        let text = strings_to_spans(&[
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ]);
        let new_text = text.replace_regex(&Regex::new("f(([aeiou])+)").unwrap(), "b${2}r");
        let target_text = strings_to_spans(&[
            Color::Red.paint("Here is some b"),
            Color::Blue.paint("or bur b"),
            Color::Green.paint("ar"),
        ]);
        println!("expected: {}", target_text);
        println!("actual:   {}", new_text);
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_across_span_simple_2_trivial() {
        let text = strings_to_spans(&[
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ]);
        let new_text = text.replace_regex(&Regex::new("f(([aeiou])+)").unwrap(), "bar");
        let target_text = strings_to_spans(&[
            Color::Red.paint("Here is some b"),
            Color::Blue.paint("ar bar b"),
            Color::Green.paint("ar"),
        ]);
        println!("expected: {}", target_text);
        println!("actual:   {}", new_text);
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_empty() {
        let text = strings_to_spans(&[
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ]);
        let new_text = text.replace_regex(&Regex::new("quux").unwrap(), "bar");
        assert_eq!(new_text, text);
    }
    #[test]
    fn replace_regex_empty_fancy() {
        let text = strings_to_spans(&[
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ]);
        let new_text = text.replace_regex(&Regex::new("([zyx])").unwrap(), "missing $1 letters");
        assert_eq!(new_text, text);
    }
    #[test]
    fn replace_regex_styled_easy() {
        let text = strings_to_spans(&[
            Color::Red.paint("Foo"),
            Color::Blue.paint("Bar"),
            Color::Green.paint("Baz"),
        ]);
        let replacement = strings_to_spans(&[Color::Cyan.paint("Quux")]);
        let actual = text.replace_regex(&Regex::new("Bar").unwrap(), &replacement);
        let expected = strings_to_spans(&[
            Color::Red.paint("Foo"),
            Color::Cyan.paint("Quux"),
            Color::Green.paint("Baz"),
        ]);
        assert_eq!(expected, actual);
    }
    #[test]
    fn replace_regex_styled_complex() {
        let text = strings_to_spans(&[
            Color::Red.paint("555"),
            Color::Black.paint("."),
            Color::Blue.paint("444"),
            Color::White.paint("."),
            Color::Green.paint("3333"),
        ]);
        let replacement = strings_to_spans(&[
            Color::Red.paint("$1"),
            Color::Black.paint("-"),
            Color::Green.paint("$2"),
            Color::Black.paint("-"),
            Color::Blue.paint("$3"),
        ]);
        let regex = Regex::new(r"(\d{3})[.-](\d{3})[.-](\d{4})").unwrap();
        for name in regex.capture_names() {
            println!("name: {:#?}", name);
        }
        println!("location: {:#?}", regex.capture_locations());
        println!("len: {:#?}", regex.capture_locations().len());
        let actual = text.replace_regex(&regex, &replacement);
        let expected = strings_to_spans(&[
            Color::Red.paint("555"),
            Color::Black.paint("-"),
            Color::Green.paint("444"),
            Color::Black.paint("-"),
            Color::Blue.paint("3333"),
        ]);
        println!("expected: {}", expected);
        println!("actual: {}", actual);
        assert_eq!(expected, actual);
    }
    #[test]
    fn span() {
        let texts = [
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ];
        let text = strings_to_spans(&texts);
        let span = text.spans().next().unwrap();
        let expected = format!("{}", texts[0]);
        let actual = format!("{}", span);
        assert_eq!(expected, actual);
    }
    #[test]
    fn raw() {
        let text = strings_to_spans(&[
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ]);

        let expected = String::from("Here is some fooo fuuu faaa");
        let actual = text.raw();
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_start() {
        let text = strings_to_spans(&[Color::Red.paint("01234"), Color::Blue.paint("56789")]);
        let actual = text.slice(0..8).unwrap();
        let expected = strings_to_spans(&[Color::Red.paint("01234"), Color::Blue.paint("567")]);

        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_middle() {
        let text = strings_to_spans(&[
            Color::Red.paint("012"),
            Color::Blue.paint("345"),
            Color::Green.paint("678"),
        ]);
        let actual = text.slice(2..8).unwrap();
        let expected = strings_to_spans(&[
            Color::Red.paint("2"),
            Color::Blue.paint("345"),
            Color::Green.paint("67"),
        ]);

        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_end() {
        let text = strings_to_spans(&[
            Color::Red.paint("012"),
            Color::Blue.paint("345"),
            Color::Green.paint("678"),
        ]);
        let actual = text.slice(2..).unwrap();
        let expected = strings_to_spans(&[
            Color::Red.paint("2"),
            Color::Blue.paint("345"),
            Color::Green.paint("678"),
        ]);

        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_full() {
        let text = strings_to_spans(&[
            Color::Red.paint("012"),
            Color::Blue.paint("345"),
            Color::Green.paint("678"),
        ]);
        let actual = text.slice(..).unwrap();
        let expected = strings_to_spans(&[
            Color::Red.paint("012"),
            Color::Blue.paint("345"),
            Color::Green.paint("678"),
        ]);

        assert_eq!(expected, actual);
    }
    #[test]
    fn split_outer() {
        let texts = vec![
            Color::Black.paint("::"),
            Color::Red.paint("Some"),
            Color::Blue.paint("::"),
            Color::Green.paint("Random"),
            Color::Cyan.paint("::"),
            Color::White.paint("Place"),
            Color::Yellow.paint("::"),
        ];
        let spans = strings_to_spans(&texts);
        let actual = spans.split("::").collect::<Vec<_>>();
        let expected = vec![
            Split {
                segment: None,
                delim: Some(string_to_spans(&texts[0])),
            },
            Split {
                segment: Some(string_to_spans(&texts[1])),
                delim: Some(string_to_spans(&texts[2])),
            },
            Split {
                segment: Some(string_to_spans(&texts[3])),
                delim: Some(string_to_spans(&texts[4])),
            },
            Split {
                segment: Some(string_to_spans(&texts[5])),
                delim: Some(string_to_spans(&texts[6])),
            },
        ];
        assert_eq!(expected, actual);
    }
    #[test]
    fn split_inner() {
        let texts = vec![
            Color::Red.paint("Some"),
            Color::Blue.paint("::"),
            Color::Green.paint("Random"),
            Color::Cyan.paint("::"),
            Color::White.paint("Place"),
        ];
        let spans = strings_to_spans(&texts);
        let actual = spans.split("::").collect::<Vec<_>>();
        let expected = vec![
            Split {
                segment: Some(string_to_spans(&texts[0])),
                delim: Some(string_to_spans(&texts[1])),
            },
            Split {
                segment: Some(string_to_spans(&texts[2])),
                delim: Some(string_to_spans(&texts[3])),
            },
            Split {
                segment: Some(string_to_spans(&texts[4])),
                delim: None,
            },
        ];
        assert_eq!(expected, actual);
    }
}
