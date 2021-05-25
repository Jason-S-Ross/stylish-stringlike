mod joinable;
mod painter;
mod repeat;
mod replaceable;
mod sliceable;
mod spans;
mod splitable;
mod width;
pub(crate) use joinable::Joinable;
pub(crate) use painter::*;
pub(crate) use repeat::*;
pub(crate) use replaceable::*;
pub(crate) use sliceable::*;
pub(crate) use spans::*;
pub(crate) use splitable::*;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
pub(crate) use width::*;

use std::ops::RangeBounds;
pub(crate) trait RawText {
    fn raw(&self) -> String;
    fn raw_ref(&self) -> &str;
}

pub(crate) trait WidthSliceable<'a> {
    type Output: 'a + Sized;
    /// Slices an object by width rather than by bytes
    fn slice_width<R>(&'a self, range: R) -> Option<Self::Output>
    where
        R: RangeBounds<usize>;
}

impl<'a, T> WidthSliceable<'a> for T
where
    T: RawText + Sliceable<'a> + 'a + Sized,
{
    type Output = T;
    /// Slices an object by width rather than by bytes
    fn slice_width<R>(&'a self, range: R) -> Option<Self::Output>
    where
        Self: Sized,
        R: RangeBounds<usize>,
    {
        let mut start_byte = None;
        let mut end_byte = None;
        let mut current_width = 0;
        let mut current_byte = 0;
        for grapheme in self.raw().graphemes(true) {
            let grapheme_width = grapheme.width();
            let in_range = {
                let mut in_range = true;
                for w in current_width..current_width + grapheme_width {
                    if !range.contains(&w) {
                        in_range = false;
                        break;
                    }
                }
                in_range
            };
            current_width += grapheme_width;
            match (in_range, start_byte) {
                (true, None) => start_byte = Some(current_byte),
                (false, Some(_)) => {
                    end_byte = Some(current_byte);
                    break;
                }
                _ => {}
            }
            current_byte += grapheme.len();
        }
        match (start_byte, end_byte) {
            (Some(s), Some(e)) => self.slice(s..e),
            (Some(s), None) => self.slice(s..),
            (None, Some(e)) => self.slice(..e),
            (None, None) => None,
        }
    }
}

pub(crate) trait HasWidth {
    fn width(&self) -> Width;
}

impl<T> HasWidth for T
where
    T: BoundedWidth,
{
    fn width(&self) -> Width {
        Width::Bounded(self.bounded_width())
    }
}

pub(crate) trait BoundedWidth {
    fn bounded_width(&self) -> usize;
}
