use ansi_term::{ANSIString, ANSIStrings, Style};
use std::fmt;
use std::iter::FromIterator;
use std::ops::{Deref, RangeBounds};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub struct StyledGrapheme<'a> {
    style: &'a Style,
    grapheme: &'a str,
}

pub trait WidthGlyph {
    fn width(&self) -> usize;
}

impl<'a> WidthGlyph for StyledGrapheme<'a> {
    fn width(&self) -> usize {
        self.grapheme.width()
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
    fn width(&'a self) -> usize {
        self.graphemes().map(|x| x.width()).sum()
    }
    fn raw(&self) -> String;
    fn slice_width<T: RangeBounds<usize> + 'a>(
        &'a self,
        range: T,
    ) -> Box<dyn Iterator<Item = G> + 'a> {
        Box::new(
            self.graphemes()
                .scan(0, move |position, g| {
                    let in_range = range.contains(position);
                    *position += g.width();
                    Some((g, in_range))
                })
                .skip_while(|(_g, in_range)| !in_range)
                .take_while(|(_g, in_range)| *in_range)
                .map(|(g, _in_range)| g),
        )
    }
}

impl<'a> Text<'a, StyledGrapheme<'a>> for Span {
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

pub struct Spans {
    spans: Vec<Span>,
}

impl<'a> Text<'a, StyledGrapheme<'a>> for Spans {
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
