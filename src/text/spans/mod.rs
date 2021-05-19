mod search_tree;
mod span;
use super::{
    slice_string, FiniteText, Graphemes, HasWidth, RawText, Replaceable, Sliceable, StyledGrapheme,
    Text, Width,
};
use ansi_term::{ANSIStrings, Style};
use regex::{Regex, Replacer};
use search_tree::SearchTree;
pub use span::Span;
use std::borrow::Borrow;
use std::borrow::Cow;
use std::convert::AsRef;
use std::fmt;
use std::iter::FromIterator;
use std::iter::{once, repeat};
use std::ops::{Add, AddAssign, RangeBounds};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Default, Debug)]
pub struct Spans<T> {
    content: String,
    /// Byte-indexed map of spans
    spans: SearchTree<usize, T>,
}

impl<T: PartialEq> Eq for Spans<T> {}

impl<T: PartialEq> PartialEq for Spans<T> {
    fn eq(&self, other: &Spans<T>) -> bool {
        self.content == other.content && self.spans == other.spans
    }
}

impl<T: Clone + Default> Spans<T> {
    #[allow(clippy::type_complexity)]
    fn segments(
        &self,
    ) -> Box<dyn Iterator<Item = ((&usize, Cow<'_, T>), Option<(&usize, Cow<'_, T>)>)> + '_> {
        if self.spans.contains_key(&0) {
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
    pub fn spans(&self) -> impl Iterator<Item = Span<'_, T>> {
        self.segments()
            .filter_map(move |((first_key, style), second)| {
                let second_key = if let Some((second_key, _)) = second {
                    *second_key
                } else {
                    self.content.len()
                };
                if let Some(ref s) = self.content.get(*first_key..second_key) {
                    Some(Span::new(style, Cow::Borrowed(s)))
                } else {
                    // This represents an invalid state in the spans.
                    // One of the spans is actually out of the range of the length of the string.
                    None
                }
            })
    }
    pub fn push(&mut self, other: &Self) -> &Self
    where
        T: PartialEq,
    {
        // copy_with_shift always succeeds because len is always positive so no
        // risk converting
        self.spans
            .copy_with_shift(&other.spans, .., self.content.len())
            .unwrap();
        self.content.push_str(&other.content);
        self
    }
    pub fn push_span(&mut self, other: &Span<'_, T>) -> &Self
    where
        T: PartialEq,
    {
        self.spans
            .insert(self.content.len(), other.style().clone().into_owned());
        self.content.push_str(&other.content());
        self.spans.dedup();
        self
    }
}

impl<'a, T: Clone + PartialEq + Default> Replaceable<'a, &'a str> for Spans<T> {
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
            result += spans;
        }
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
                result += spans;
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
            result += spans;
        }
        result
    }
}

impl<'a, T: Clone + PartialEq + Default> Replaceable<'a, &'a Spans<T>> for Spans<T> {
    fn replace(&'a self, from: &str, replacer: &'a Spans<T>) -> Self {
        let mut result = Spans {
            content: String::new(),
            spans: SearchTree::new(),
        };

        let mut last_end = 0;
        for (start, part) in self.content.match_indices(from) {
            if let Some(spans) = self.slice(last_end..start) {
                result.push(&spans);
                result.push(replacer);
            }
            last_end = start + part.len();
        }
        if let Some(spans) = self.slice(last_end..) {
            result += spans;
        }
        result
    }
    fn replace_regex(&'a self, searcher: &Regex, replacer: &'a Spans<T>) -> Self {
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
                result += spans;
                if let Some(_original) = self.slice(mat.start()..mat.end()) {
                    for span in replacer.spans() {
                        let mut dst = String::new();
                        capture.expand(span.content(), &mut dst);
                        let new_span = Span::<T>::borrowed(&span.style(), &dst);
                        result.push_span(&new_span);
                    }
                }
                last_end = mat.end();
            }
        }
        if let Some(spans) = self.slice(last_end..) {
            result += spans;
        }
        result.spans.trim(result.content.len() - 1);
        result
    }
}

impl<'a, T: Clone> Sliceable<'a> for Spans<T> {
    type Output = Spans<T>;
    type Index = usize;
    fn slice<R>(&'a self, range: R) -> Option<Self::Output>
    where
        R: RangeBounds<Self::Index> + Clone,
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

impl<'a, T> Graphemes<'a, T> for Spans<T>
where
    T: Clone + Default,
{
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a, T>> + 'a> {
        Box::new(
            self.content
                .grapheme_indices(true)
                .map(move |(start_byte, grapheme)| {
                    if let Some(style) = self.spans.search_left(&start_byte) {
                        StyledGrapheme::<T>::borrowed(style, grapheme)
                    } else {
                        StyledGrapheme::<T>::owned(Default::default(), String::from(grapheme))
                    }
                }),
        )
    }
}

impl<'a, T> FromIterator<StyledGrapheme<'a, T>> for Spans<T>
where
    T: Clone + PartialEq + Default + 'a,
{
    fn from_iter<I>(iter: I) -> Spans<T>
    where
        I: IntoIterator<Item = StyledGrapheme<'a, T>>,
    {
        let mut content = String::new();
        let mut last_style: Option<Cow<T>> = None;
        let mut spans = SearchTree::<usize, T>::new();
        for grapheme in iter {
            let len = content.len();
            match last_style {
                Some(ref style) if style.as_ref() == grapheme.style().as_ref() => {}
                _ => {
                    if let Some(_style) = spans.insert(len, grapheme.style().as_ref().clone()) {
                        unreachable!("Failed to insert {:#?} into tree", len)
                    }
                    last_style = Some(grapheme.style().clone())
                }
            }
            content.push_str(&grapheme.grapheme());
        }
        Spans { content, spans }
    }
}

impl<'a, T, U> FromIterator<U> for Spans<T>
where
    T: Clone + PartialEq + 'a + Default,
    U: Borrow<Spans<T>> + 'a,
{
    fn from_iter<I>(iter: I) -> Spans<T>
    where
        I: IntoIterator<Item = U>,
    {
        let mut result: Spans<T> = Default::default();
        for span in iter {
            result += span.borrow().clone()
        }
        result
    }
}

impl<'a, T> FromIterator<Span<'a, T>> for Spans<T>
where
    T: Clone + PartialEq + Default + 'a,
{
    fn from_iter<I>(iter: I) -> Spans<T>
    where
        I: IntoIterator<Item = Span<'a, T>>,
    {
        let mut result: Spans<T> = Default::default();
        for span in iter {
            result.push_span(&span);
        }
        result
    }
}

impl<T: Clone + Default> HasWidth for Spans<T> {
    fn width(&self) -> Width {
        self.graphemes().map(|x| x.width()).sum()
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

impl<'a, T: Clone + Default + 'a> Text<'a, T> for Spans<T> {}

impl<'a, S, T> From<&'a S> for Spans<T>
where
    S: Graphemes<'a, T> + 'a,
    T: Clone + 'a + Default + PartialEq,
{
    fn from(iter: &'a S) -> Spans<T> {
        iter.graphemes().collect()
    }
}

impl<T> From<&str> for Spans<T>
where
    T: Clone + Default + PartialEq,
{
    fn from(other: &str) -> Spans<T> {
        let mut spans: SearchTree<_, _> = Default::default();
        spans.insert(0, Default::default());
        Spans {
            content: String::from(other),
            spans,
        }
    }
}

impl fmt::Display for Spans<Style> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        ANSIStrings(&self.spans().map(|span| (span).into()).collect::<Vec<_>>()).fmt(fmt)
    }
}

impl<T> Add for Spans<T>
where
    T: Clone + Default + PartialEq,
{
    type Output = Spans<T>;
    fn add(self, other: Self) -> Self::Output {
        vec![self, other].iter().collect()
    }
}

impl<T> Add for &Spans<T>
where
    T: Clone + PartialEq + Default,
{
    type Output = Spans<T>;
    fn add(self, other: Self) -> Self::Output {
        vec![self, other].iter().cloned().collect()
    }
}

impl<T> AddAssign for Spans<T>
where
    T: Clone + PartialEq,
{
    fn add_assign(&mut self, rhs: Self) {
        // copy_with_shift always succeeds because len is always positive so no
        // risk converting
        self.spans
            .copy_with_shift(&rhs.spans, .., self.content.len())
            .unwrap();
        self.content.push_str(&rhs.content);
    }
}

impl<'a, T: Clone + Default + 'a> FiniteText<'a, T> for Spans<T> {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::{Split, Splitable};
    use ansi_term::{ANSIStrings, Color};
    use std::ops::Bound;
    #[test]
    fn test_slice_width() {
        let texts = vec![Color::Green.paint("foo")];
        let text: Spans<_> = (&texts).into();
        let actual: Spans<_> = text
            .slice_width((Bound::Unbounded, Bound::Included(2)))
            .collect();
        let texts2 = vec![Color::Green.paint("fo")];
        let expected: Spans<_> = (&texts2).into();
        assert_eq!(expected, actual);
    }
    #[test]
    fn test_slice_width_hard() {
        let texts = vec![Color::Green.paint("👱👱👱")];
        let text: Spans<_> = (&texts).into();
        let actual: Spans<_> = text
            .slice_width((Bound::Unbounded, Bound::Included(3)))
            .collect();
        let texts2 = vec![Color::Green.paint("👱")];
        let expected: Spans<_> = (&texts2).into();
        assert_eq!(expected, actual);
        let actual: Spans<_> = text
            .slice_width((Bound::Unbounded, Bound::Included(4)))
            .collect();
        let texts2 = vec![Color::Green.paint("👱👱")];
        let expected: Spans<_> = (&texts2).into();
        assert_eq!(expected, actual);
    }
    #[test]
    fn test_finite_width() {
        let texts = vec![Color::Green.paint("foo")];
        let text: Spans<_> = (&texts).into();
        let expected = 3;
        let actual = text.bounded_width();
        assert_eq!(expected, actual);
    }
    #[test]
    fn build_span() {
        let texts = vec![Color::Green.paint("foo")];
        let text: Spans<_> = (&texts).into();
        let string = ANSIStrings(&texts);
        assert_eq!(format!("{}", text), format!("{}", string));
    }
    #[test]
    fn build_spans() {
        let texts = vec![
            Color::Red.paint("a"),
            Color::Blue.paint("b"),
            Color::Blue.paint("⛇"),
        ];
        let text: Spans<_> = (&texts).into();
        let string = ANSIStrings(&texts);
        assert_eq!(format!("{}", text), format!("{}", string));
    }
    #[test]
    fn simple_replace() {
        let texts = vec![Color::Red.paint("foo")];
        let text: Spans<_> = (&texts).into();
        let new_text = text.replace("foo", "bar");
        let target_texts = vec![Color::Red.paint("bar")];
        let target_text: Spans<_> = (&target_texts).into();

        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_in_span() {
        let texts = vec![Color::Red.paint("Bob "), Color::Blue.paint("Dylan")];
        let text: Spans<_> = (&texts).into();
        let new_text = text.replace("Bob", "Robert");
        let target_texts = vec![Color::Red.paint("Robert "), Color::Blue.paint("Dylan")];
        let target_text: Spans<_> = (&target_texts).into();
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_chars() {
        let texts = vec![
            Color::Blue.paint("what"),
            Color::Red.paint("//\\/;,!"),
            Color::Blue.paint("the fudge"),
        ];
        let text: Spans<_> = (&texts).into();
        let new_text = text.replace("/", "/");
        let target_texts = vec![
            Color::Blue.paint("what"),
            Color::Red.paint("//\\/;,!"),
            Color::Blue.paint("the fudge"),
        ];
        let target_text: Spans<_> = (&target_texts).into();
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_across_span_simple_2() {
        let texts = vec![
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("oo foo fo"),
            Color::Green.paint("o"),
        ];
        let text: Spans<_> = (&texts).into();
        let new_text = text.replace("foo", "bar");
        let target_texts = vec![
            Color::Red.paint("Here is some b"),
            Color::Blue.paint("ar bar ba"),
            Color::Green.paint("r"),
        ];
        let target_text: Spans<_> = (&target_texts).into();
        eprintln!("expected: {}", target_text);
        eprintln!("actual: {}", new_text);
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn simple_regex_replace() {
        let texts = vec![Color::Red.paint("foooo")];
        let text: Spans<_> = (&texts).into();
        let new_text = text.replace_regex(&Regex::new("fo+").unwrap(), "bar");
        let target_texts = vec![Color::Red.paint("bar")];
        let target_text: Spans<_> = (&target_texts).into();

        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_across_span_simple_trival() {
        let texts = vec![Color::Red.paint("Here lies "), Color::Blue.paint("Beavis")];
        let text: Spans<_> = (&texts).into();
        let new_text = text.replace_regex(
            &Regex::new(r"(Here lies) Beavis").unwrap(),
            "Here lies Butthead",
        );
        let target_texts = vec![
            Color::Red.paint("Here lies "),
            Color::Blue.paint("Butthead"),
        ];
        let target_text: Spans<_> = (&target_texts).into();
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_across_span_simple_backref() {
        let texts = vec![Color::Red.paint("Here lies "), Color::Blue.paint("Beavis")];
        let text: Spans<_> = (&texts).into();
        let new_text =
            text.replace_regex(&Regex::new(r"(Here lies) Beavis").unwrap(), "$1 Butthead");
        let target_texts = vec![
            Color::Red.paint("Here lies "),
            Color::Blue.paint("Butthead"),
        ];
        let target_text: Spans<_> = (&target_texts).into();
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_across_span_simple_2_backref() {
        let texts = vec![
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ];
        let text: Spans<_> = (&texts).into();
        let new_text = text.replace_regex(&Regex::new("f(([aeiou])+)").unwrap(), "b${2}r");
        let target_texts = vec![
            Color::Red.paint("Here is some b"),
            Color::Blue.paint("or bur b"),
            Color::Green.paint("ar"),
        ];
        let target_text: Spans<_> = (&target_texts).into();
        println!("expected: {}", target_text);
        println!("actual:   {}", new_text);
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_across_span_simple_2_trivial() {
        let texts = vec![
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ];
        let text: Spans<_> = (&texts).into();
        let new_text = text.replace_regex(&Regex::new("f(([aeiou])+)").unwrap(), "bar");
        let target_texts = vec![
            Color::Red.paint("Here is some b"),
            Color::Blue.paint("ar bar b"),
            Color::Green.paint("ar"),
        ];
        let target_text: Spans<_> = (&target_texts).into();
        println!("expected: {}", target_text);
        println!("actual:   {}", new_text);
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_empty() {
        let texts = vec![
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ];
        let text: Spans<_> = (&texts).into();
        let new_text = text.replace_regex(&Regex::new("quux").unwrap(), "bar");
        assert_eq!(new_text, text);
    }
    #[test]
    fn replace_regex_empty_fancy() {
        let texts = vec![
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ];
        let text: Spans<_> = (&texts).into();
        let new_text = text.replace_regex(&Regex::new("([zyx])").unwrap(), "missing $1 letters");
        assert_eq!(new_text, text);
    }
    #[test]
    fn replace_regex_styled_easy() {
        let texts = vec![
            Color::Red.paint("Foo"),
            Color::Blue.paint("Bar"),
            Color::Green.paint("Baz"),
        ];
        let text: Spans<_> = (&texts).into();
        let replacement: Spans<_> = (&Color::Cyan.paint("Quux")).into();
        let actual = text.replace_regex(&Regex::new("Bar").unwrap(), &replacement);
        let expected: Spans<_> = (&vec![
            Color::Red.paint("Foo"),
            Color::Cyan.paint("Quux"),
            Color::Green.paint("Baz"),
        ])
            .into();
        assert_eq!(expected, actual);
    }
    #[test]
    fn replace_regex_styled_complex() {
        let text: Spans<_> = (&vec![
            Color::Red.paint("555"),
            Color::Black.paint("."),
            Color::Blue.paint("444"),
            Color::White.paint("."),
            Color::Green.paint("3333"),
        ])
            .into();
        let replacement: Spans<_> = (&vec![
            Color::Red.paint("$1"),
            Color::Black.paint("-"),
            Color::Green.paint("$2"),
            Color::Black.paint("-"),
            Color::Blue.paint("$3"),
        ])
            .into();
        let foo = "555.444.3333";
        let regex = Regex::new(r"(\d{3})[.-](\d{3})[.-](\d{4})").unwrap();
        for name in regex.capture_names() {
            println!("name: {:#?}", name);
        }
        println!("location: {:#?}", regex.capture_locations());
        println!("len: {:#?}", regex.capture_locations().len());
        let actual = text.replace_regex(&regex, &replacement);
        let expected: Spans<_> = (&vec![
            Color::Red.paint("555"),
            Color::Black.paint("-"),
            Color::Green.paint("444"),
            Color::Black.paint("-"),
            Color::Blue.paint("3333"),
        ])
            .into();
        println!("expected: {}", expected);
        println!("actual: {}", actual);
        assert_eq!(expected, actual);
    }
    #[test]
    fn span() {
        let texts = vec![
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ];
        let text: Spans<_> = (&texts).into();
        let span = text.spans().next().unwrap();
        let expected = format!("{}", texts[0]);
        let actual = format!("{}", span);
        assert_eq!(expected, actual);
    }
    #[test]
    fn graphemes() {
        let texts = vec![
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ];
        let text: Spans<_> = (&texts).into();
        let s = "H";
        let style = Color::Red.normal();
        let expected = StyledGrapheme::borrowed(&style, s);
        let actual = text.graphemes().next().unwrap();
        assert_eq!(expected, actual);
    }
    #[test]
    fn raw() {
        let texts = vec![
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ];
        let text: Spans<_> = (&texts).into();

        let expected = String::from("Here is some fooo fuuu faaa");
        let actual = text.raw();
        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_start() {
        let texts = vec![Color::Red.paint("01234"), Color::Blue.paint("56789")];
        let text: Spans<_> = (&texts).into();
        let actual = text.slice(0..8).unwrap();
        let texts = vec![Color::Red.paint("01234"), Color::Blue.paint("567")];
        let expected: Spans<_> = (&texts).into();

        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_middle() {
        let texts = vec![
            Color::Red.paint("012"),
            Color::Blue.paint("345"),
            Color::Green.paint("678"),
        ];
        let text: Spans<_> = (&texts).into();
        let actual = text.slice(2..8).unwrap();
        let texts = vec![
            Color::Red.paint("2"),
            Color::Blue.paint("345"),
            Color::Green.paint("67"),
        ];
        let expected: Spans<_> = (&texts).into();

        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_end() {
        let texts = vec![
            Color::Red.paint("012"),
            Color::Blue.paint("345"),
            Color::Green.paint("678"),
        ];
        let text: Spans<_> = (&texts).into();
        let actual = text.slice(2..).unwrap();
        let texts = vec![
            Color::Red.paint("2"),
            Color::Blue.paint("345"),
            Color::Green.paint("678"),
        ];
        let expected: Spans<_> = (&texts).into();

        assert_eq!(expected, actual);
    }
    #[test]
    fn slice_full() {
        let texts = vec![
            Color::Red.paint("012"),
            Color::Blue.paint("345"),
            Color::Green.paint("678"),
        ];
        let text: Spans<_> = (&texts).into();
        let actual = text.slice(..).unwrap();
        let texts = vec![
            Color::Red.paint("012"),
            Color::Blue.paint("345"),
            Color::Green.paint("678"),
        ];
        let expected: Spans<_> = (&texts).into();

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
        let spans: Spans<_> = (&texts).into();
        let actual = spans.split_style("::").collect::<Vec<_>>();
        let expected = vec![
            Split {
                segment: None,
                delim: Some(Spans::from(&texts[0])),
            },
            Split {
                segment: Some(Spans::from(&texts[1])),
                delim: Some(Spans::from(&texts[2])),
            },
            Split {
                segment: Some(Spans::from(&texts[3])),
                delim: Some(Spans::from(&texts[4])),
            },
            Split {
                segment: Some(Spans::from(&texts[5])),
                delim: Some(Spans::from(&texts[6])),
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
        let spans: Spans<_> = (&texts).into();
        let actual = spans.split_style("::").collect::<Vec<_>>();
        let expected = vec![
            Split {
                segment: Some(Spans::from(&texts[0])),
                delim: Some(Spans::from(&texts[1])),
            },
            Split {
                segment: Some(Spans::from(&texts[2])),
                delim: Some(Spans::from(&texts[3])),
            },
            Split {
                segment: Some(Spans::from(&texts[4])),
                delim: None,
            },
        ];
        assert_eq!(expected, actual);
    }
}
