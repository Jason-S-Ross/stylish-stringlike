mod search_tree;
mod span;
use super::{FiniteText, Graphemes, HasWidth, StyledGrapheme, Text, Width};
use ansi_term::{ANSIStrings, Style};
use regex::{Regex, Replacer};
use search_tree::SearchTree;
pub use span::Span;
use std::fmt;
use std::iter::FromIterator;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Default, Debug)]
pub struct Spans {
    content: String,
    /// Byte-indexed map of spans
    spans: SearchTree<usize, Style>,
    default_style: Style,
}

#[cfg(test)]
impl Eq for Spans {}

#[cfg(test)]
impl PartialEq for Spans {
    fn eq(&self, other: &Spans) -> bool {
        self.content == other.content
            && self.spans == other.spans
            && self.default_style == other.default_style
    }
}

impl Spans {
    #[allow(dead_code)]
    pub fn replace(&self, from: &str, to: &str) -> Result<Self, ()> {
        let mut result = String::new();
        let mut spans = SearchTree::<usize, Style>::new();
        let mut last_start = 0;
        let mut last_end = 0;
        let mut shift_total: isize = 0;
        let shift_incr: isize = to.len() as isize - from.len() as isize;

        for (start, part) in self.content.match_indices(from) {
            result.push_str(&self.content[last_end..start]);
            result.push_str(to);
            let end = start + part.len();
            spans.copy_with_shift(&self.spans, last_start..=end, shift_total)?;
            shift_total += shift_incr;
            last_end = end;
            last_start = start;
        }
        result.push_str(&self.content[last_end..]);
        spans.copy_with_shift(&self.spans, last_end.., shift_total)?;
        Ok(Spans {
            content: result,
            spans,
            ..*self
        })
    }
    #[allow(dead_code)]
    pub fn replace_regex<R>(&self, searcher: &Regex, mut replacer: R) -> Result<Self, ()>
    where
        R: Replacer,
    {
        // Implement the same strategy as regex but it's a pain

        pub struct Replacement {
            pub start: usize,
            pub end: usize,
            pub to: String,
        }
        let replacements: Box<dyn Iterator<Item = Replacement>> = {
            use std::iter;
            if let Some(replacer) = replacer.no_expansion() {
                let mut matches = searcher.find_iter(&self.content).peekable();
                if matches.peek().is_none() {
                    Box::new(iter::empty())
                } else {
                    Box::new(matches.map(move |mat| Replacement {
                        start: mat.start(),
                        end: mat.end(),
                        to: String::from(replacer.clone()),
                    }))
                }
            } else {
                let mut captures = searcher.captures_iter(&self.content).peekable();
                if captures.peek().is_none() {
                    Box::new(iter::empty())
                } else {
                    Box::new(captures.map(move |capture| {
                        let mat = capture
                            .get(0)
                            .expect("Failed to unwrap capture 0. Possible api change to regex");
                        let mut to = String::new();
                        replacer.replace_append(&capture, &mut to);
                        Replacement {
                            start: mat.start(),
                            end: mat.end(),
                            to,
                        }
                    }))
                }
            }
        };
        let mut spans = SearchTree::<usize, Style>::new();
        let mut result = String::with_capacity(self.content.len());
        let mut last_end = 0;
        let mut shift_total: isize = 0;
        for repl in replacements {
            let start = repl.start;
            let end = repl.end;
            let match_len = end - start;
            let to = repl.to;
            let shift_incr = to.len() as isize - match_len as isize;
            spans.copy_with_shift(&self.spans, last_end..start, shift_total)?;
            result.push_str(&self.content[last_end..start]);
            result.push_str(&to);
            spans.copy_with_shift(&self.spans, start..end, shift_total)?;
            shift_total += shift_incr;
            last_end = end;
        }

        result.push_str(&self.content[last_end..]);
        spans.copy_with_shift(&self.spans, last_end.., shift_total)?;
        Ok(Spans {
            content: result,
            spans,
            ..*self
        })
    }
    pub fn spans(&self) -> impl Iterator<Item = Span<'_>> + '_ {
        self.spans
            .iter()
            .zip(
                self.spans
                    .iter()
                    .map(Some)
                    .skip(1)
                    .chain(std::iter::repeat(None)),
            )
            .filter_map(move |((first_key, style), second)| {
                let second_key = if let Some((second_key, _)) = second {
                    *second_key
                } else {
                    self.content.len()
                };
                if let Some(ref s) = self.content.get(*first_key..second_key) {
                    Some(Span::borrowed(style, s))
                } else {
                    // This represents an invalid state in the spans.
                    // One of the spans is actually out of the range of the length of the string.
                    None
                }
            })
    }
}

impl<'a> Graphemes<'a> for Spans {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        Box::new(
            self.content
                .grapheme_indices(true)
                .map(move |(start_byte, grapheme)| {
                    let style = if let Some(ref s) = self.spans.search_left(&start_byte) {
                        s
                    } else {
                        &self.default_style
                    };
                    StyledGrapheme::borrowed(style, grapheme)
                }),
        )
    }
}

impl<'a> FromIterator<StyledGrapheme<'a>> for Spans {
    fn from_iter<I>(iter: I) -> Spans
    where
        I: IntoIterator<Item = StyledGrapheme<'a>>,
    {
        let mut content = String::new();
        let mut last_style: Option<Style> = None;
        let mut spans = SearchTree::<usize, Style>::new();
        for grapheme in iter {
            let len = content.len();
            match last_style {
                Some(style) if &style == grapheme.style().as_ref() => {}
                _ => {
                    if let Some(_style) = spans.insert(len, **grapheme.style()) {
                        unreachable!("Failed to insert {:#?} into tree {:#?}", len, spans)
                    }
                    last_style = Some(**grapheme.style())
                }
            }
            content.push_str(&grapheme.grapheme());
        }
        Spans {
            content,
            spans,
            default_style: Default::default(),
        }
    }
}

impl HasWidth for Spans {
    fn width(&self) -> Width {
        self.graphemes().map(|x| x.width()).sum()
    }
}

impl<'a> Text<'a> for Spans {
    fn raw(&self) -> String {
        self.content.clone()
    }
}

impl<'a, T> From<&'a T> for Spans
where
    T: Graphemes<'a> + 'a,
{
    fn from(iter: &'a T) -> Spans {
        iter.graphemes().collect()
    }
}

impl fmt::Display for Spans {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        ANSIStrings(&self.spans().map(|span| (span).into()).collect::<Vec<_>>()).fmt(fmt)
    }
}

impl<'a> FiniteText<'a> for Spans {}

#[cfg(test)]
mod test {
    use super::*;
    use ansi_term::{ANSIStrings, Color};
    use std::ops::Bound;
    #[test]
    fn test_slice_width() {
        let texts = vec![Color::Green.paint("foo")];
        let text: Spans = (&texts).into();
        let actual: Spans = text
            .slice_width((Bound::Unbounded, Bound::Included(2)))
            .collect();
        let texts2 = vec![Color::Green.paint("fo")];
        let expected: Spans = (&texts2).into();
        assert_eq!(expected, actual);
    }
    #[test]
    fn test_slice_width_hard() {
        let texts = vec![Color::Green.paint("👱👱👱")];
        let text: Spans = (&texts).into();
        let actual: Spans = text
            .slice_width((Bound::Unbounded, Bound::Included(3)))
            .collect();
        let texts2 = vec![Color::Green.paint("👱")];
        let expected: Spans = (&texts2).into();
        assert_eq!(expected, actual);
        let actual: Spans = text
            .slice_width((Bound::Unbounded, Bound::Included(4)))
            .collect();
        let texts2 = vec![Color::Green.paint("👱👱")];
        let expected: Spans = (&texts2).into();
        assert_eq!(expected, actual);
    }
    #[test]
    fn test_finite_width() {
        let texts = vec![Color::Green.paint("foo")];
        let text: Spans = (&texts).into();
        let expected = 3;
        let actual = text.bounded_width();
        assert_eq!(expected, actual);
    }
    #[test]
    fn build_span() {
        let texts = vec![Color::Green.paint("foo")];
        let text: Spans = (&texts).into();
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
        let text: Spans = (&texts).into();
        let string = ANSIStrings(&texts);
        assert_eq!(format!("{}", text), format!("{}", string));
    }
    #[test]
    fn simple_replace() {
        let texts = vec![Color::Red.paint("foo")];
        let text: Spans = (&texts).into();
        let new_text = text.replace("foo", "bar").unwrap();
        let target_texts = vec![Color::Red.paint("bar")];
        let target_text: Spans = (&target_texts).into();

        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_in_span() {
        let texts = vec![Color::Red.paint("Bob "), Color::Blue.paint("Dylan")];
        let text: Spans = (&texts).into();
        let new_text = text.replace("Bob", "Robert").unwrap();
        let target_texts = vec![Color::Red.paint("Robert "), Color::Blue.paint("Dylan")];
        let target_text: Spans = (&target_texts).into();
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_across_span_simple() {
        let texts = vec![Color::Red.paint("Here lies "), Color::Blue.paint("Beavis")];
        let text: Spans = (&texts).into();
        let new_text = text
            .replace("Here lies Beavis", "Here lies Butthead")
            .unwrap();
        let target_texts = vec![
            Color::Red.paint("Here lies "),
            Color::Blue.paint("Butthead"),
        ];
        let target_text: Spans = (&target_texts).into();
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_across_span_simple_2() {
        let texts = vec![
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("oo foo fo"),
            Color::Green.paint("o"),
        ];
        let text: Spans = (&texts).into();
        let new_text = text.replace("foo", "bar").unwrap();
        let target_texts = vec![
            Color::Red.paint("Here is some b"),
            Color::Blue.paint("ar bar ba"),
            Color::Green.paint("r"),
        ];
        let target_text: Spans = (&target_texts).into();
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn simple_regex_replace() {
        let texts = vec![Color::Red.paint("foooo")];
        let text: Spans = (&texts).into();
        let new_text = text
            .replace_regex(&Regex::new("fo+").unwrap(), "bar")
            .unwrap();
        let target_texts = vec![Color::Red.paint("bar")];
        let target_text: Spans = (&target_texts).into();

        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_across_span_simple_trival() {
        let texts = vec![Color::Red.paint("Here lies "), Color::Blue.paint("Beavis")];
        let text: Spans = (&texts).into();
        let new_text = text
            .replace_regex(
                &Regex::new(r"(Here lies) Beavis").unwrap(),
                "Here lies Butthead",
            )
            .unwrap();
        let target_texts = vec![
            Color::Red.paint("Here lies "),
            Color::Blue.paint("Butthead"),
        ];
        let target_text: Spans = (&target_texts).into();
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_across_span_simple_backref() {
        let texts = vec![Color::Red.paint("Here lies "), Color::Blue.paint("Beavis")];
        let text: Spans = (&texts).into();
        let new_text = text
            .replace_regex(&Regex::new(r"(Here lies) Beavis").unwrap(), "$1 Butthead")
            .unwrap();
        let target_texts = vec![
            Color::Red.paint("Here lies "),
            Color::Blue.paint("Butthead"),
        ];
        let target_text: Spans = (&target_texts).into();
        assert_eq!(new_text, target_text);
    }
    #[test]
    fn replace_regex_across_span_simple_2_backref() {
        let texts = vec![
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ];
        let text: Spans = (&texts).into();
        let new_text = text
            .replace_regex(&Regex::new("f(([aeiou])+)").unwrap(), "b${2}r")
            .unwrap();
        let target_texts = vec![
            Color::Red.paint("Here is some b"),
            Color::Blue.paint("or bur b"),
            Color::Green.paint("ar"),
        ];
        let target_text: Spans = (&target_texts).into();
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
        let text: Spans = (&texts).into();
        let new_text = text
            .replace_regex(&Regex::new("f(([aeiou])+)").unwrap(), "bar")
            .unwrap();
        let target_texts = vec![
            Color::Red.paint("Here is some b"),
            Color::Blue.paint("ar bar b"),
            Color::Green.paint("ar"),
        ];
        let target_text: Spans = (&target_texts).into();
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
        let text: Spans = (&texts).into();
        let new_text = text
            .replace_regex(&Regex::new("quux").unwrap(), "bar")
            .unwrap();
        assert_eq!(new_text, text);
    }
    #[test]
    fn replace_regex_empty_fancy() {
        let texts = vec![
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ];
        let text: Spans = (&texts).into();
        let new_text = text
            .replace_regex(&Regex::new("([zyx])").unwrap(), "missing $1 letters")
            .unwrap();
        assert_eq!(new_text, text);
    }
    #[test]
    fn span() {
        let texts = vec![
            Color::Red.paint("Here is some f"),
            Color::Blue.paint("ooo fuuu f"),
            Color::Green.paint("aaa"),
        ];
        let text: Spans = (&texts).into();
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
        let text: Spans = (&texts).into();
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
        let text: Spans = (&texts).into();

        let expected = String::from("Here is some fooo fuuu faaa");
        let actual = text.raw();
        assert_eq!(expected, actual);
    }
}
