use ansi_term::{ANSIString, Style};
use ouroboros::self_referencing;
use std::fmt;
use std::iter::{FromIterator, Sum};
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

impl<'a> StyledGrapheme<'a> {
    pub fn width(&self) -> Width {
        Width::Bounded(self.grapheme.width())
    }
    pub fn raw(&self) -> String {
        self.grapheme.to_owned()
    }
}

impl<'a> fmt::Display for StyledGrapheme<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.style.paint(self.grapheme).fmt(fmt)
    }
}

pub struct Span<'a> {
    style: Style,
    content: &'a str,
}

impl<'a> fmt::Display for Span<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.style.paint(self.content).fmt(fmt)
    }
}

pub trait Graphemes<'a> {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a>;
}

pub trait Text<'a>: fmt::Display + Graphemes<'a> {
    fn width(&'a self) -> Width;
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

pub trait FiniteText<'a>: Text<'a> {
    fn bounded_width(&'a self) -> usize {
        match self.width() {
            Width::Bounded(w) => w,
            Width::Unbounded => {
                unreachable!("Created a finite text object with an unbounded width")
            }
        }
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

impl<'a> Text<'a> for Span<'a> {
    fn width(&'a self) -> Width {
        self.graphemes().map(|x| x.width()).sum()
    }
    fn raw(&self) -> String {
        self.content.to_owned()
    }
}

impl<'a> FiniteText<'a> for Span<'a> {}

#[self_referencing]
pub struct Spans {
    content: String,
    #[borrows(content)]
    #[covariant]
    spans: Vec<Span<'this>>,
}

impl<'a> Graphemes<'a> for Spans {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        Box::new(self.borrow_spans().iter().flat_map(|x| x.graphemes()))
    }
}

impl<'a> FromIterator<StyledGrapheme<'a>> for Spans {
    fn from_iter<I>(iter: I) -> Spans
    where
        I: IntoIterator<Item = StyledGrapheme<'a>>,
    {
        #[derive(Debug)]
        struct SpanMarker {
            style: Style,
            start: usize,
            end: usize,
        }
        let mut content = String::new();
        let mut span_markers: Vec<SpanMarker> = vec![];
        let mut start = 0;
        let mut last_style: Option<Style> = None;
        for grapheme in iter {
            let len = content.len();
            match last_style {
                Some(style) if style != *grapheme.style => {
                    span_markers.push(SpanMarker {
                        style,
                        start,
                        end: len,
                    });
                    start = len;
                    last_style = Some(*grapheme.style)
                }
                Some(_style) => {}
                None => last_style = Some(*grapheme.style),
            }
            content.push_str(&grapheme.grapheme);
        }
        if let Some(style) = last_style {
            span_markers.push(SpanMarker {
                style,
                start,
                end: content.len(),
            });
        }
        SpansBuilder {
            content,
            spans_builder: |s: &str| {
                span_markers
                    .iter()
                    .map(|marker| Span {
                        content: &s[marker.start..marker.end],
                        style: marker.style,
                    })
                    .collect()
            },
        }
        .build()
    }
}

impl<'a> Text<'a> for Spans {
    fn width(&'a self) -> Width {
        self.graphemes().map(|x| x.width()).sum()
    }
    fn raw(&self) -> String {
        self.borrow_content().to_owned()
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
        for span in self.borrow_spans() {
            match span.fmt(fmt) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

impl<'a> FiniteText<'a> for Spans {}

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
