use crate::text::{FiniteText, HasWidth, StyledGrapheme, Text, Width};
use std::fmt;
use std::ops::Bound;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum TruncationStyle {
    #[allow(dead_code)]
    Left,
    #[allow(dead_code)]
    Right,
    #[allow(dead_code)]
    Inner,
    #[allow(dead_code)]
    Outer,
}

#[allow(dead_code)]
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
                Bound::Included(width.saturating_sub(symbol.bounded_width())),
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
                Bound::Included(width.saturating_sub(symbol.bounded_width())),
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
                    .chain(self.slice_width((Bound::Excluded(start), Bound::Included(end))))
                    .chain(symbol.graphemes()),
            )
        } else {
            Box::new(
                symbol
                    .graphemes()
                    .chain(
                        self.slice_width((
                            Bound::Unbounded,
                            Bound::Included(width - 2 * sym_width),
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
        let w = text_width / 2;
        if let Width::Bounded(self_width) = self_width {
            Box::new(
                self.slice_width((Bound::Unbounded, Bound::Included(w + text_width % 2)))
                    .chain(symbol.graphemes())
                    .chain(self.slice_width((
                        Bound::Excluded(self_width.saturating_sub(w)),
                        Bound::Unbounded,
                    ))),
            )
        } else {
            Box::new(
                self.slice_width((Bound::Unbounded, Bound::Included(w + text_width % 2)))
                    .chain(symbol.graphemes())
                    .chain(self.slice_width((Bound::Unbounded, Bound::Included(w)))),
            )
        }
    }
}
