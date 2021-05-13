pub use self::spans::{Span, Spans};
pub use self::width::Width;
use ansi_term::{ANSIString, Style};
use std::fmt;
use std::ops::{Bound, Deref, RangeBounds};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Debug)]
pub struct StyledGrapheme<'a> {
    style: &'a Style,
    grapheme: &'a str,
}

impl<'a> StyledGrapheme<'a> {
    pub fn raw(&self) -> String {
        self.grapheme.to_owned()
    }
}

impl<'a> HasWidth for StyledGrapheme<'a> {
    fn width(&self) -> Width {
        Width::Bounded(self.grapheme.width())
    }
}

impl<'a> fmt::Display for StyledGrapheme<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.style.paint(self.grapheme).fmt(fmt)
    }
}

pub mod width {
    use std::iter::Sum;
    use std::ops::{Add, AddAssign};

    #[derive(Copy, Clone)]
    pub enum Width {
        Bounded(usize),
        Unbounded,
    }

    impl Add for Width {
        type Output = Width;
        fn add(self, other: Self) -> Self::Output {
            use Width::{Bounded, Unbounded};
            match (self, other) {
                (Unbounded, _) | (_, Unbounded) => Unbounded,
                (Bounded(left), Bounded(right)) => Bounded(left + right),
            }
        }
    }

    impl AddAssign for Width {
        fn add_assign(&mut self, other: Self) {
            use Width::{Bounded, Unbounded};
            *self = match (*self, other) {
                (Unbounded, _) | (_, Unbounded) => Unbounded,
                (Bounded(left), Bounded(right)) => Bounded(left + right),
            };
        }
    }

    impl Sum for Width {
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.fold(Width::Bounded(0), |a, b| a + b)
        }
    }
}

pub trait HasWidth {
    fn width(&self) -> Width;
}

pub trait Graphemes<'a> {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a>;
}

pub trait Text<'a>: fmt::Display + Graphemes<'a> + HasWidth {
    fn raw(&self) -> String;
    fn slice_width(
        &'a self,
        range: (Bound<usize>, Bound<usize>),
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        Box::new(
            self.graphemes()
                .scan(0, move |position, g| {
                    let in_range = range.contains(position);
                    if let Width::Bounded(w) = g.width() {
                        *position += w;
                        Some((g, in_range))
                    } else {
                        None
                    }
                })
                .skip_while(|(_g, in_range)| !in_range)
                .take_while(|(_g, in_range)| *in_range)
                .map(|(g, _in_range)| g),
        )
    }
}

pub trait FiniteText<'a>: Text<'a> + fmt::Debug {
    fn bounded_width(&'a self) -> usize {
        match self.width() {
            Width::Bounded(w) => w,
            Width::Unbounded => {
                unreachable!("Created a finite text object with an unbounded width")
            }
        }
    }
}

pub mod spans {

    use super::*;
    use ansi_term::{ANSIStrings, Style};
    use regex::{Regex, Replacer};
    use std::iter::FromIterator;

    /// Contains a data structure to allow fast lookup of the value to the left.
    mod search_tree {
        use std::borrow::Borrow;
        use std::collections::btree_map::Iter;
        use std::collections::btree_map::Range;
        use std::collections::BTreeMap;
        use std::convert::TryFrom;
        use std::ops::{Add, RangeBounds};
        /// Data structure to quickly look up the nearest value smaller than a given value.
        #[derive(Clone, Debug, Default, Eq, PartialEq)]
        pub struct SearchTree<K, V>
        where
            K: Ord,
        {
            tree: BTreeMap<K, V>,
        }
        impl<K, V> SearchTree<K, V>
        where
            K: Ord,
        {
            pub fn new() -> SearchTree<K, V>
            where
                K: Ord,
            {
                SearchTree {
                    tree: BTreeMap::<K, V>::new(),
                }
            }
            pub fn range<T, R>(&self, range: R) -> Range<'_, K, V>
            where
                T: Ord + ?Sized,
                R: RangeBounds<T>,
                K: Borrow<T> + Ord,
            {
                self.tree.range(range)
            }
            pub fn search_left<T>(&self, key: &T) -> Option<&V>
            where
                T: Ord,
                K: Borrow<T> + Ord,
            {
                if let Some(ref v) = self.tree.get(key) {
                    Some(v)
                } else if let Some((_last_key, ref v)) = self.tree.range(..key).last() {
                    Some(v)
                } else {
                    None
                }
            }
            pub fn insert(&mut self, key: K, value: V) -> Option<V>
            where
                K: Ord,
            {
                self.tree.insert(key, value)
            }
            pub fn iter(&self) -> Iter<K, V> {
                self.tree.iter()
            }
            #[allow(dead_code)]
            pub(super) fn keys(&self) -> Vec<K>
            where
                K: Clone,
            {
                self.tree.keys().cloned().collect()
            }
            /// Drops keys that have the same value as the previous keys
            fn dedup(&mut self)
            where
                V: PartialEq,
                K: Clone,
            {
                let drop_keys: Vec<_> = self
                    .tree
                    .iter()
                    .zip(self.tree.iter().skip(1))
                    .filter_map(|((_first_key, first_val), (second_key, second_val))| {
                        if first_val == second_val {
                            Some(second_key)
                        } else {
                            None
                        }
                    })
                    .cloned()
                    .collect();
                for key in drop_keys {
                    self.tree.remove(&key);
                }
            }
            /// Copy values in a range from another tree into this tree,
            /// shifting the keys by some amount.
            pub(super) fn copy_with_shift<T, R, S>(
                &mut self,
                from: &SearchTree<K, V>,
                range: R,
                shift: S,
            ) -> Result<(), ()>
            where
                V: Clone + PartialEq,
                T: Ord + ?Sized,
                R: RangeBounds<T>,
                K: Borrow<T> + Ord + TryFrom<S> + Copy,
                S: Add<Output = S> + TryFrom<K> + Copy,
            {
                let contained_spans = from.range(range);
                for (key, value) in contained_spans {
                    if let Ok(add_key) = S::try_from(*key) {
                        if let Ok(new_key) = K::try_from(add_key + shift) {
                            self.insert(new_key, value.clone());
                        } else {
                            self.insert(*key, value.clone());
                        }
                    } else {
                        return Err(());
                    }
                }
                self.dedup();
                Ok(())
            }
        }
    }

    /// Contains a data structure to represent a segment of a Spans object
    pub mod span {
        use super::*;
        use ansi_term::ANSIString;
        #[derive(Debug)]
        pub struct Span<'a> {
            style: &'a Style,
            content: &'a str,
        }
        impl<'a> fmt::Display for Span<'a> {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                self.style.paint(self.content).fmt(fmt)
            }
        }
        impl<'a> Graphemes<'a> for Span<'a> {
            fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
                Box::new(
                    self.content
                        .graphemes(true)
                        .map(move |grapheme| StyledGrapheme {
                            style: &self.style,
                            grapheme,
                        }),
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
                self.content.to_owned()
            }
        }
        impl<'a> FiniteText<'a> for Span<'a> {}
        impl<'a> Span<'a> {
            pub fn new(style: &'a Style, content: &'a str) -> Span<'a> {
                Span { style, content }
            }
        }
        impl<'a> From<&'a Span<'a>> for ANSIString<'a> {
            fn from(span: &'a Span<'a>) -> ANSIString<'a> {
                span.style.paint(span.content)
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
                let span = Span::new(&style, "foo");
                let s: ANSIString = (&span).into();
            }
        }
    }

    pub use span::Span;

    use search_tree::SearchTree;

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
                        Some(Span::new(style, s))
                    } else {
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
                        StyledGrapheme { grapheme, style }
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
                    Some(style) if style == *grapheme.style => {}
                    _ => {
                        if let Some(_style) = spans.insert(len, *grapheme.style) {
                            unreachable!("Failed to insert {:#?} into tree {:#?}", len, spans)
                        }
                        last_style = Some(*grapheme.style)
                    }
                }
                content.push_str(&grapheme.grapheme);
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
    }
}

impl<'a> Graphemes<'a> for ANSIString<'a> {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        Box::new(
            self.deref()
                .graphemes(true)
                .map(move |grapheme| StyledGrapheme {
                    style: self.style_ref(),
                    grapheme,
                }),
        )
    }
}

impl<'a> Graphemes<'a> for Vec<ANSIString<'a>> {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        Box::new(self.iter().flat_map(move |s| {
            let style = s.style_ref();
            s.deref()
                .graphemes(true)
                .map(move |grapheme| StyledGrapheme { style, grapheme })
        }))
    }
}
