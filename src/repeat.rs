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
}
