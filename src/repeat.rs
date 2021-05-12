use crate::text::{Graphemes, HasWidth, StyledGrapheme, Text, Width};
use std::fmt;
use std::iter::repeat;
use std::ops::{Bound, RangeBounds};

#[derive(Debug)]
pub struct Repeat<'a> {
    grapheme: StyledGrapheme<'a>,
}

impl<'a> Repeat<'a> {
    pub fn new(grapheme: StyledGrapheme<'a>) -> Self {
        Repeat { grapheme }
    }
}

impl<'a> fmt::Display for Repeat<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.grapheme.fmt(fmt)
    }
}

impl<'a> Graphemes<'a> for Repeat<'a> {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        Box::new(repeat(self.grapheme.clone()))
    }
}

impl<'a> HasWidth for Repeat<'a> {
    fn width(&self) -> Width {
        Width::Unbounded
    }
}

impl<'a> Text<'a> for Repeat<'a> {
    fn raw(&self) -> String {
        self.grapheme.raw()
    }
    fn slice_width(
        &'a self,
        range: (Bound<usize>, Bound<usize>),
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        if let Width::Bounded(width) = self.grapheme.width() {
            Box::new(self.graphemes().scan(0, move |position, g| {
                let in_range = range.contains(position);
                if in_range {
                    *position += width;
                    Some(g)
                } else {
                    None
                }
            }))
        } else {
            Box::new(std::iter::Empty::default())
        }
    }
}
