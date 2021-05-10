use crate::text::{Text, WidthGlyph};
use std::fmt;
use std::marker::PhantomData;
use std::ops::Bound;

pub trait Truncatable<'a, G>: fmt::Display
where
    G: WidthGlyph + 'a,
{
    fn truncate_left(
        &'a self,
        width: usize,
        symbol: &'a dyn Text<'a, G>,
    ) -> Box<dyn Iterator<Item = G> + 'a>;
    fn truncate_right(
        &'a self,
        width: usize,
        symbol: &'a dyn Text<'a, G>,
    ) -> Box<dyn Iterator<Item = G> + 'a>;
    fn truncate_outer(
        &'a self,
        width: usize,
        symbol: &'a dyn Text<'a, G>,
    ) -> Box<dyn Iterator<Item = G> + 'a>;
    fn truncate_inner(
        &'a self,
        width: usize,
        symbol: &'a dyn Text<'a, G>,
    ) -> Box<dyn Iterator<Item = G> + 'a>;
}

#[derive(Copy, Clone)]
pub enum TruncationStyle {
    Left,
    Right,
    Inner,
    Outer,
}

pub struct TextWidget<'a, G>
where
    G: WidthGlyph + 'a,
{
    text: &'a dyn Truncatable<'a, G>,
    truncation_style: TruncationStyle,
    truncation_symbol: &'a dyn Text<'a, G>,
}

impl<'a, G> TextWidget<'a, G>
where
    G: WidthGlyph + 'a,
{
    pub fn new(
        text: &'a dyn Truncatable<'a, G>,
        truncation_style: TruncationStyle,
        truncation_symbol: &'a dyn Text<'a, G>,
    ) -> Self {
        TextWidget {
            text,
            truncation_style,
            truncation_symbol,
        }
    }
    fn truncate(&self, width: usize) -> Box<dyn Iterator<Item = G> + 'a> {
        use TruncationStyle::{Inner, Left, Outer, Right};
        match self.truncation_style {
            Left => self.text.truncate_left(width, self.truncation_symbol),
            Right => self.text.truncate_right(width, self.truncation_symbol),
            Inner => self.text.truncate_inner(width, self.truncation_symbol),
            Outer => self.text.truncate_outer(width, self.truncation_symbol),
        }
    }
}

pub struct HBox<'a, G>
where
    G: WidthGlyph + 'a,
{
    elements: Vec<&'a TextWidget<'a, G>>,
    _g: PhantomData<G>,
}

impl<'a, G> HBox<'a, G>
where
    G: WidthGlyph + 'a,
{
    pub fn new(elements: &[&'a TextWidget<'a, G>]) -> Self {
        HBox {
            elements: elements.to_vec(),
            _g: Default::default(),
        }
    }
    pub fn truncate(&'a self, width: usize) -> Box<dyn Iterator<Item = G> + 'a> {
        let w = width / self.elements.len();
        let rem = width % self.elements.len();
        Box::new(
            self.elements
                .iter()
                .enumerate()
                .flat_map(move |(i, widget)| {
                    let w = if i < rem { w + 1 } else { w };
                    widget.truncate(w)
                }),
        )
    }
}

impl<'a, G, T> Truncatable<'a, G> for T
where
    T: Text<'a, G>,
    G: WidthGlyph + 'a,
{
    fn truncate_left(
        &'a self,
        width: usize,
        symbol: &'a dyn Text<'a, G>,
    ) -> Box<dyn Iterator<Item = G> + 'a> {
        let self_width = self.width();
        if width >= self_width {
            return self.graphemes();
        }
        Box::new(
            self.slice_width((
                Bound::Unbounded,
                Bound::Excluded(width.saturating_sub(symbol.width())),
            ))
            .chain(symbol.graphemes()),
        )
    }
    fn truncate_right(
        &'a self,
        width: usize,
        symbol: &'a dyn Text<'a, G>,
    ) -> Box<dyn Iterator<Item = G> + 'a> {
        let self_width = self.width();
        if width >= self_width {
            return self.graphemes();
        }
        Box::new(
            symbol.graphemes().chain(
                self.slice_width((
                    Bound::Excluded(
                        self.width()
                            .saturating_sub(width.saturating_sub(symbol.width())),
                    ),
                    Bound::Unbounded,
                )),
            ),
        )
    }
    fn truncate_outer(
        &'a self,
        width: usize,
        symbol: &'a dyn Text<'a, G>,
    ) -> Box<dyn Iterator<Item = G> + 'a> {
        let self_width = self.width();
        if width >= self_width {
            return self.graphemes();
        }
        let diff = self_width.saturating_sub(width) + 2 * symbol.width();
        let start = diff / 2;
        let end = start + width.saturating_sub(2 * symbol.width());
        Box::new(
            symbol
                .graphemes()
                .chain(self.slice_width((Bound::Excluded(start), Bound::Excluded(end))))
                .chain(symbol.graphemes()),
        )
    }
    fn truncate_inner(
        &'a self,
        width: usize,
        symbol: &'a dyn Text<'a, G>,
    ) -> Box<dyn Iterator<Item = G> + 'a> {
        let self_width = self.width();
        if width >= self_width {
            return self.graphemes();
        }
        let text_width = width.saturating_sub(symbol.width());
        let w = text_width / 2 + text_width % 2;
        Box::new(
            self.slice_width((Bound::Unbounded, Bound::Excluded(w)))
                .chain(symbol.graphemes())
                .chain(self.slice_width((
                    Bound::Excluded(self.width().saturating_sub(w) + text_width % 2),
                    Bound::Unbounded,
                ))),
        )
    }
}
