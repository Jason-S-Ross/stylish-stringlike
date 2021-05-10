use ansi_term::{ANSIString, ANSIStrings, Style};
use std::fmt;
use std::iter::{FromIterator, Sum};
use std::ops::{Add, AddAssign, Bound, Deref, RangeBounds};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone)]
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

pub trait WidthGlyph: fmt::Display + Clone {
    fn width(&self) -> Width;
    fn raw(&self) -> String;
}

impl<'a> WidthGlyph for StyledGrapheme<'a> {
    fn width(&self) -> Width {
        Width::Bounded(self.grapheme.width())
    }
    fn raw(&self) -> String {
        self.grapheme.to_owned()
    }
}

impl<'a> fmt::Display for StyledGrapheme<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.style.paint(self.grapheme).fmt(fmt)
    }
}

pub struct Span {
    style: Style,
    content: String,
}

impl fmt::Display for Span {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.style.paint(&self.content).fmt(fmt)
    }
}

impl<'a> From<&'a ANSIString<'a>> for Span {
    fn from(s: &'a ANSIString<'a>) -> Self {
        Span {
            style: *s.style_ref(),
            content: s.deref().to_string(),
        }
    }
}

impl<'a> From<&'a Span> for ANSIString<'a> {
    fn from(span: &'a Span) -> Self {
        span.style.paint(&span.content)
    }
}

pub trait Text<'a, G: 'a>: fmt::Display
where
    G: WidthGlyph,
{
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = G> + 'a>;
    fn width(&'a self) -> Width;
    fn raw(&self) -> String;
    fn slice_width(
        &'a self,
        range: (Bound<usize>, Bound<usize>),
    ) -> Box<dyn Iterator<Item = G> + 'a> {
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

pub trait FiniteText<'a, G: 'a>: Text<'a, G>
where
    G: WidthGlyph,
{
    fn bounded_width(&'a self) -> usize {
        match self.width() {
            Width::Bounded(w) => w,
            Width::Unbounded => {
                unreachable!("Created a finite text object with an unbounded width")
            }
        }
    }
}

impl<'a> Text<'a, StyledGrapheme<'a>> for Span {
    fn width(&'a self) -> Width {
        self.graphemes().map(|x| x.width()).sum()
    }
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
    fn raw(&self) -> String {
        self.content.to_owned()
    }
}

impl<'a> FiniteText<'a, StyledGrapheme<'a>> for Span {}

pub struct Spans {
    spans: Vec<Span>,
}

impl<'a> Text<'a, StyledGrapheme<'a>> for Spans {
    fn width(&'a self) -> Width {
        self.graphemes().map(|x| x.width()).sum()
    }
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        Box::new(self.spans.iter().flat_map(|x| x.graphemes()))
    }
    fn raw(&self) -> String {
        self.spans.iter().map(|x| x.raw()).collect()
    }
}

impl<'a> FromIterator<StyledGrapheme<'a>> for Spans {
    fn from_iter<I: IntoIterator<Item = StyledGrapheme<'a>>>(iter: I) -> Spans {
        let mut spans: Vec<Span> = vec![];
        for grapheme in iter {
            match spans.iter_mut().last() {
                Some(span) if span.style == *grapheme.style => {
                    span.content.push_str(&grapheme.grapheme)
                }
                _ => spans.push(Span {
                    style: *grapheme.style,
                    content: grapheme.grapheme.to_string(),
                }),
            }
        }
        Spans { spans }
    }
}

impl fmt::Display for Spans {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut strings = vec![];
        for span in &self.spans {
            strings.push(ANSIString::from(span));
        }
        let ansistrings = ANSIStrings(&strings);
        ansistrings.fmt(fmt)
    }
}

impl<'a> FiniteText<'a, StyledGrapheme<'a>> for Spans {}
