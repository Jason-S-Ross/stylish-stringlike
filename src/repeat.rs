use crate::text::{Text, Width, WidthGlyph};
use std::fmt;
use std::iter::repeat;
use std::ops::{Bound, RangeBounds};

pub struct Repeat<G>
where
    G: WidthGlyph,
{
    grapheme: G,
}

impl<G> Repeat<G>
where
    G: WidthGlyph,
{
    pub fn new(grapheme: G) -> Self {
        Repeat { grapheme }
    }
}

impl<G> fmt::Display for Repeat<G>
where
    G: WidthGlyph,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.grapheme.fmt(fmt)
    }
}

impl<'a, G> Text<'a, G> for Repeat<G>
where
    G: WidthGlyph + 'a,
{
    fn width(&'a self) -> Width {
        Width::Unbounded
    }
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = G> + 'a> {
        Box::new(repeat(self.grapheme.clone()))
    }
    fn raw(&self) -> String {
        self.grapheme.raw()
    }
    fn slice_width(
        &'a self,
        range: (Bound<usize>, Bound<usize>),
    ) -> Box<dyn Iterator<Item = G> + 'a> {
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
