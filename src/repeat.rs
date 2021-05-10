use crate::text::{Text, WidthGlyph};
use std::fmt;
use std::iter::repeat;

pub struct Repeat<G>
where
    G: WidthGlyph,
{
    grapheme: G,
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
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = G> + 'a> {
        Box::new(repeat(self.grapheme.clone()))
    }
    fn raw(&self) -> String {
        self.grapheme.raw()
    }
}
