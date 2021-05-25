use crate::text::{BoundedWidth, HasWidth, Width, WidthSliceable};
use std::fmt::Display;
use std::ops::Bound;

pub(crate) trait Truncateable<'a>: HasWidth + WidthSliceable<'a> {}

impl<'a, T> Truncateable<'a> for T
where
    T: WidthSliceable<'a> + HasWidth,
    T::Output: Display + Sized,
{
}

pub(crate) trait TruncationStrategy<'a, T>
where
    T: WidthSliceable<'a> + HasWidth,
    T::Output: Display,
{
    fn truncate(&'a self, target: &'a T, width: usize) -> Option<String>;
}

pub(crate) enum TruncationStyle<T: BoundedWidth + Display> {
    #[allow(dead_code)]
    Left(Option<T>),
    #[allow(dead_code)]
    Right(Option<T>),
    #[allow(dead_code)]
    Inner(Option<T>),
}

impl<'a, T, S: BoundedWidth + Display> TruncationStrategy<'a, T> for TruncationStyle<S>
where
    T: WidthSliceable<'a> + HasWidth,
    T::Output: Display,
{
    fn truncate(&'a self, target: &'a T, width: usize) -> Option<String> {
        use TruncationStyle::*;
        let w = match target.width() {
            Width::Bounded(w) if width >= w => return Some(format!("{}", target.slice_width(..)?)),
            Width::Bounded(w) => w,
            Width::Unbounded => {
                return match self {
                    Left(Some(symbol)) => {
                        let slice =
                            target.slice_width(..width.saturating_sub(symbol.bounded_width()));
                        match slice {
                            Some(text) => Some(format!("{}{}", text, symbol)),
                            None => Some(format!("{}", symbol)),
                        }
                    }
                    Left(None) | Right(None) => {
                        let slice = target.slice_width(..width);
                        slice.map(|text| format!("{}", text))
                    }
                    Right(Some(symbol)) => {
                        let slice =
                            target.slice_width(..width.saturating_sub(symbol.bounded_width()));
                        match slice {
                            Some(text) => Some(format!("{}{}", symbol, text)),
                            None => Some(format!("{}", symbol)),
                        }
                    }
                    Inner(s) => {
                        let inner_width = if let Some(s) = s {
                            s.bounded_width()
                        } else {
                            0
                        };
                        let target_width = width.saturating_sub(inner_width);
                        let left_width = target_width / 2 + target_width % 2;
                        let right_width = target_width / 2;
                        eprintln!("left_width: {}, right_width: {}", left_width, right_width);
                        let left_slice = target.slice_width(..left_width);
                        let right_slice = target.slice_width(..right_width);
                        match (s, left_slice, right_slice) {
                            (Some(s), Some(left), Some(right)) => {
                                Some(format!("{}{}{}", left, s, right))
                            }
                            (None, Some(left), Some(right)) => Some(format!("{}{}", left, right)),
                            (Some(s), Some(left), None) => Some(format!("{}{}", left, s)),
                            (None, Some(left), None) => Some(format!("{}", left)),
                            (Some(s), None, Some(right)) => Some(format!("{}{}", s, right)),
                            (None, None, Some(right)) => Some(format!("{}", right)),
                            (Some(s), None, None) => Some(format!("{}", s)),
                            (None, None, None) => None,
                        }
                    }
                }
            }
        };
        if let Inner(s) = self {
            let inner_width = if let Some(s) = s {
                s.bounded_width()
            } else {
                0
            };
            let target_width = width.saturating_sub(inner_width);
            let left_width = target_width / 2 + target_width % 2;
            let right_width = target_width / 2;
            let left_slice = target.slice_width(..left_width);
            let right_slice = target.slice_width(w.saturating_sub(right_width)..);
            match (s, left_slice, right_slice) {
                (Some(s), Some(left), Some(right)) => Some(format!("{}{}{}", left, s, right)),
                (None, Some(left), Some(right)) => Some(format!("{}{}", left, right)),
                (Some(s), Some(left), None) => Some(format!("{}{}", left, s)),
                (None, Some(left), None) => Some(format!("{}", left)),
                (Some(s), None, Some(right)) => Some(format!("{}{}", s, right)),
                (None, None, Some(right)) => Some(format!("{}", right)),
                (Some(s), None, None) => Some(format!("{}", s)),
                (None, None, None) => None,
            }
        } else {
            let slice = match self {
                Left(Some(symbol)) => (
                    Bound::Unbounded,
                    Bound::Excluded(width.saturating_sub(symbol.bounded_width())),
                ),
                Left(None) => (Bound::Unbounded, Bound::Included(width)),
                Right(Some(symbol)) => (
                    Bound::Included(w.saturating_sub(width.saturating_sub(symbol.bounded_width()))),
                    Bound::Unbounded,
                ),
                Right(None) => (Bound::Included(w.saturating_sub(width)), Bound::Unbounded),
                _ => unreachable!("Already caught the inner case"),
            };
            let sliced = target.slice_width(slice);
            match (self, sliced) {
                (Left(Some(sym)), None) | (Right(Some(sym)), None) => Some(format!("{}", sym)),
                (Left(None), Some(txt)) | (Right(None), Some(txt)) => Some(format!("{}", txt)),
                (Left(Some(sym)), Some(txt)) => Some(format!("{}{}", txt, sym)),
                (Right(Some(sym)), Some(txt)) => Some(format!("{}{}", sym, txt)),
                (Left(None), None) | (Right(None), None) => None,
                _ => unreachable!("Already caught the inner case"),
            }
        }
    }
}
