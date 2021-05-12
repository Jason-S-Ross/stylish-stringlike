use crate::text::{FiniteText, HasWidth, StyledGrapheme, Text, Width};
use std::fmt;
use std::ops::Bound;

pub trait Truncatable<'a>: fmt::Display + HasWidth + fmt::Debug {
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

#[derive(Copy, Clone, Debug)]
pub enum TruncationStyle {
    Left,
    Right,
    Inner,
    Outer,
}

#[derive(Debug)]
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

impl<'a, T> Truncatable<'a> for T
where
    T: Text<'a> + fmt::Debug,
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
        let mut space = width;
        let mut todo: Vec<(usize, _)> = self
            .elements
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                if let Width::Bounded(_w) = element.text.width() {
                    Some((index, element))
                } else {
                    None
                }
            })
            .collect();
        let mut to_fit = todo.len();
        let mut widths: std::collections::HashMap<usize, usize> = Default::default();
        while to_fit > 0 {
            let target_width: f32 = space as f32 / to_fit as f32;
            let mut to_pop = vec![];
            for (rel_index, (index, element)) in todo.iter().enumerate() {
                if let Width::Bounded(w) = element.text.width() {
                    if (w as f32) <= target_width {
                        space -= w;
                        to_fit -= 1;
                        widths.insert(*index, w);
                        to_pop.push(rel_index)
                    }
                }
            }
            for index in to_pop.iter().rev() {
                todo.remove(*index);
            }
            if to_pop.is_empty() {
                let target_width = space / todo.len();
                let rem = space % todo.len();
                for (i, (index, _widget)) in todo.iter().enumerate() {
                    let w = if i < rem {
                        target_width + 1
                    } else {
                        target_width
                    };
                    space -= w;
                    widths.insert(*index, w);
                }
                break;
            }
        }
        let infinite_widths: Vec<(usize, _)> = self
            .elements
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                if let Width::Unbounded = element.text.width() {
                    Some((index, element))
                } else {
                    None
                }
            })
            .collect();
        {
            let target_width = space / infinite_widths.len();
            let rem = space % infinite_widths.len();
            for (rel_index, (abs_index, _element)) in infinite_widths.iter().enumerate() {
                let w = if rel_index < rem {
                    target_width + 1
                } else {
                    target_width
                };
                widths.insert(*abs_index, w);
            }
        }

        Box::new(
            self.elements
                .iter()
                .enumerate()
                .flat_map(move |(i, widget)| widget.truncate(widths[&i])),
        )
    }
}
