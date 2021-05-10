use crate::text::{FiniteText, StyledGrapheme, Text, Width};
use std::fmt;
use std::marker::PhantomData;
use std::ops::Bound;

pub trait Truncatable<'a>: fmt::Display {
    fn truncate_left(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a>;
    fn truncate_right(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a>;
    fn truncate_outer(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a>;
    fn truncate_inner(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a>;
}

#[derive(Copy, Clone)]
pub enum TruncationStyle {
    Left,
    Right,
    Inner,
    Outer,
}

pub struct TextWidget<'a> {
    text: &'a dyn Truncatable<'a>,
    truncation_style: TruncationStyle,
    truncation_symbol: &'a dyn FiniteText<'a>,
}

impl<'a> TextWidget<'a> {
    pub fn new(
        text: &'a dyn Truncatable<'a>,
        truncation_style: TruncationStyle,
        truncation_symbol: &'a dyn FiniteText<'a>,
    ) -> Self {
        TextWidget {
            text,
            truncation_style,
            truncation_symbol,
        }
    }
    fn truncate(&self, width: usize) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        use TruncationStyle::{Inner, Left, Outer, Right};
        match self.truncation_style {
            Left => self.text.truncate_left(width, self.truncation_symbol),
            Right => self.text.truncate_right(width, self.truncation_symbol),
            Inner => self.text.truncate_inner(width, self.truncation_symbol),
            Outer => self.text.truncate_outer(width, self.truncation_symbol),
        }
    }
}

pub struct HBox<'a> {
    elements: Vec<&'a TextWidget<'a>>,
}

impl<'a> HBox<'a> {
    pub fn new(elements: &[&'a TextWidget<'a>]) -> Self {
        HBox {
            elements: elements.to_vec(),
        }
    }
    pub fn truncate(&'a self, width: usize) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
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

impl<'a, T> Truncatable<'a> for T
where
    T: Text<'a>,
{
    fn truncate_left(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        let self_width = self.width();
        match self_width {
            Width::Bounded(w) if width >= w => {
                return self.graphemes();
            }
            _ => {}
        }
        Box::new(
            self.slice_width((
                Bound::Unbounded,
                Bound::Excluded(width.saturating_sub(symbol.bounded_width())),
            ))
            .chain(symbol.graphemes()),
        )
    }
    fn truncate_right(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        let self_width = self.width();
        match self_width {
            Width::Bounded(w) if width >= w => {
                return self.graphemes();
            }
            _ => {}
        }
        if let Width::Bounded(self_width) = self_width {
            Box::new(symbol.graphemes().chain(self.slice_width((
                Bound::Excluded(
                    self_width.saturating_sub(width.saturating_sub(symbol.bounded_width())),
                ),
                Bound::Unbounded,
            ))))
        } else {
            Box::new(symbol.graphemes().chain(self.slice_width((
                Bound::Unbounded,
                Bound::Excluded(width.saturating_sub(symbol.bounded_width())),
            ))))
        }
    }
    fn truncate_outer(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        let self_width = self.width();
        match self_width {
            Width::Bounded(w) if width >= w => {
                return self.graphemes();
            }
            _ => {}
        }
        let sym_width = symbol.bounded_width();
        if let Width::Bounded(self_width) = self_width {
            let diff = self_width.saturating_sub(width) + 2 * sym_width;
            let start = diff / 2;
            let end = start + width.saturating_sub(2 * sym_width);
            Box::new(
                symbol
                    .graphemes()
                    .chain(self.slice_width((Bound::Excluded(start), Bound::Excluded(end))))
                    .chain(symbol.graphemes()),
            )
        } else {
            Box::new(
                symbol
                    .graphemes()
                    .chain(
                        self.slice_width((
                            Bound::Unbounded,
                            Bound::Excluded(width - 2 * sym_width),
                        )),
                    )
                    .chain(symbol.graphemes()),
            )
        }
    }
    fn truncate_inner(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        let self_width = self.width();
        match self_width {
            Width::Bounded(w) if width >= w => {
                return self.graphemes();
            }
            _ => {}
        }
        let sym_width = symbol.bounded_width();
        let text_width = width.saturating_sub(sym_width);
        let w = text_width / 2 + text_width % 2;
        if let Width::Bounded(self_width) = self_width {
            Box::new(
                self.slice_width((Bound::Unbounded, Bound::Excluded(w)))
                    .chain(symbol.graphemes())
                    .chain(self.slice_width((
                        Bound::Excluded(self_width.saturating_sub(w) + text_width % 2),
                        Bound::Unbounded,
                    ))),
            )
        } else {
            Box::new(
                self.slice_width((Bound::Unbounded, Bound::Excluded(w)))
                    .chain(symbol.graphemes())
                    .chain(self.slice_width((Bound::Unbounded, Bound::Excluded(w)))),
            )
        }
    }
}
