pub use self::spans::Spans;
use ansi_term::{ANSIString, Style};
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Bound, Deref, RangeBounds};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Debug)]
pub struct StyledGrapheme<'a> {
    style: &'a Style,
    grapheme: &'a str,
}

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

pub trait HasWidth {
    fn width(&self) -> Width;
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
    use ansi_term::Style;
    use std::iter::FromIterator;

    /// Contains a data structure to allow fast lookup of the value to the left.
    mod search_tree {
        use std::borrow::Borrow;
        use std::collections::btree_map::Iter;
        use std::collections::BTreeMap;
        /// Data structure to quickly look up the nearest value smaller than a given value.
        #[derive(Debug)]
        pub struct SearchTree<K, V> {
            tree: BTreeMap<K, V>,
        }
        impl<K, V> SearchTree<K, V> {
            pub fn new() -> SearchTree<K, V>
            where
                K: Ord,
            {
                SearchTree {
                    tree: BTreeMap::<K, V>::new(),
                }
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
        }
    }

    /// Contains a data structure to represent a segment of a Spans object
    pub mod span {
        use super::*;
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
    }

    pub use span::Span;

    use search_tree::SearchTree;

    #[derive(Debug)]
    pub struct Spans {
        content: String,
        /// Byte-indexed map of spans
        spans: SearchTree<usize, Style>,
        default_style: Style,
    }

    impl Spans {
        pub fn spans(&self) -> impl Iterator<Item = Span<'_>> {
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
                default_style: Style::new(),
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
        // TODO: This is a lazy implementation. It would be more efficient
        // to build an ANSIStrings from this instead of sending all the
        // extra characters to the terminal
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            for grapheme in self.graphemes() {
                match grapheme.fmt(fmt) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        }
    }

    impl<'a> FiniteText<'a> for Spans {}
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
