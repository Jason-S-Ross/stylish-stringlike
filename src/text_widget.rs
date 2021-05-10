use std::marker::PhantomData;
use crate::text::{Span, Spans, StyledGrapheme, Text, WidthGlyph};
use std::fmt;

pub trait Truncatable<'a, G>: fmt::Display
where
    G: WidthGlyph + 'a,
{
    fn truncate_left<S>(&'a self, width: usize, symbol: &'a S) -> Box<dyn Iterator<Item = G> + 'a>
        where S: Text<'a, G>;
    fn truncate_right<S>(&'a self, width: usize, symbol: &'a S) -> Box<dyn Iterator<Item = G> + 'a>
        where S: Text<'a, G>;
    fn truncate_outer<S>(&'a self, width: usize, symbol: &'a S) -> Box<dyn Iterator<Item = G> + 'a>
        where S: Text<'a, G>;
    fn truncate_inner<S>(&'a self, width: usize, symbol: &'a S) -> Box<dyn Iterator<Item = G> + 'a>
        where S: Text<'a, G>;
}

pub trait TextWidget<'a, G>: Truncatable<'a, G>
where
    G: WidthGlyph + 'a
{}

pub struct HBox<'a, G, T>
    where
    T: Truncatable<'a, G>,
    G: WidthGlyph + 'a
{
    elements: Vec<&'a T>,
    _g: PhantomData<G>
}


impl<'a, G, T> Truncatable<'a, G> for T
where
    T: Text<'a, G>,
    G: WidthGlyph + 'a
{
    fn truncate_left<S>(
        &'a self,
        width: usize,
        symbol: &'a S,
    ) -> Box<dyn Iterator<Item = G> + 'a>
        where S: Text<'a, G>
    {
        let self_width = self.width();
        if width >= self_width {
            return self.graphemes()
        }
        Box::new(
            self.slice_width(..(width.saturating_sub(symbol.width())))
                .chain(symbol.graphemes()),
        )
    }
    fn truncate_right<S>(
        &'a self,
        width: usize,
        symbol: &'a S,
    ) -> Box<dyn Iterator<Item = G> + 'a>
        where S: Text<'a, G>
    {
        let self_width = self.width();
        if width >= self_width {
            return self.graphemes()
        }
        Box::new(
            symbol.graphemes().chain(
             self.slice_width(self.width().saturating_sub(width.saturating_sub(symbol.width()))..)
            )
        )
    }
    fn truncate_outer<S>(
        &'a self,
        width: usize,
        symbol: &'a S,
    ) -> Box<dyn Iterator<Item = G> + 'a>
        where S: Text<'a, G>
    {
        let self_width = self.width();
        if width >= self_width {
            return self.graphemes()
        }
        let diff = self_width.saturating_sub(width) + 2 * symbol.width();
        let start = diff / 2 ;
        let end = start + width.saturating_sub(2 * symbol.width()) ;
        Box::new(
            symbol.graphemes().chain(
                self.slice_width(start..end)
            ).chain(symbol.graphemes())
        )
    }
    fn truncate_inner<S>(
        &'a self,
        width: usize,
        symbol: &'a S,
    ) -> Box<dyn Iterator<Item = G> + 'a>
        where S: Text<'a, G>
    {
        let self_width = self.width();
        if width >= self_width {
            return self.graphemes()
        }
        let text_width = width.saturating_sub(symbol.width());
        let w = text_width/ 2 + text_width % 2;
        Box::new(
            self.slice_width(..w)
                .chain(symbol.graphemes())
                .chain(self.slice_width(self.width().saturating_sub(w) + text_width % 2..))
        )
    }
}
