use crate::text::{Graphemes, HasWidth, RawText, StyledGrapheme, Text, Width};
use ansi_term::Style;
use std::fmt;
use std::iter::repeat;

#[derive(Debug)]
pub struct Repeat<'a, T>
where
    T: Clone,
{
    grapheme: StyledGrapheme<'a, T>,
}

impl<'a, T> Repeat<'a, T>
where
    T: Clone,
{
    #[allow(dead_code)]
    pub fn new(grapheme: StyledGrapheme<'a, T>) -> Repeat<T> {
        Repeat { grapheme }
    }
}

impl<'a> fmt::Display for Repeat<'a, Style> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.grapheme.fmt(fmt)
    }
}

impl<'a, T> Graphemes<'a, T> for Repeat<'a, T>
where
    T: Clone,
{
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a, T>> + 'a> {
        Box::new(repeat(self.grapheme.clone()))
    }
}

impl<'a, T: Clone> HasWidth for Repeat<'a, T> {
    fn width(&self) -> Width {
        Width::Unbounded
    }
}

impl<'a, T: Clone> Text<'a, T> for Repeat<'a, T> {}

impl<'a, T> RawText for Repeat<'a, T>
where
    T: Clone,
{
    fn raw(&self) -> String {
        self.grapheme.raw()
    }
    fn raw_ref<'b>(&self) -> &str {
        self.grapheme.raw_ref()
    }
}
