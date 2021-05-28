use crate::text::{RawText, Sliceable};
use std::ops::RangeBounds;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Provides a function for slicing by grapheme width rather than bytes.
///
/// This is useful for ensuring that a text object fits in a given terminal
/// width.
pub trait WidthSliceable {
    type Output: Sized;
    /// Slice an object by width rather than by bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use stylish_stringlike::text::WidthSliceable;
    /// let foo = String::from("foobar");
    /// assert_eq!(Some(String::from("oob")), foo.slice_width(1..4));
    /// let bar = String::from("🙈🙉🙊");
    /// // Monkeys are two columns wide, so we get nothing back
    /// assert_eq!(None, bar.slice_width(..1));
    /// // We get one monkey for two columns
    /// assert_eq!(Some(String::from("🙈")), bar.slice_width(..2));
    /// // If we aren't column-aligned, we get nothing because no one monkey fits between 1 and 3
    /// assert_eq!(None, bar.slice_width(1..3));
    /// ```
    fn slice_width<R>(&self, range: R) -> Option<Self::Output>
    where
        R: RangeBounds<usize>;
}

impl<T> WidthSliceable for T
where
    T: RawText + Sliceable + Sized,
{
    type Output = T;
    fn slice_width<R>(&self, range: R) -> Option<Self::Output>
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

impl<T> WidthSliceable for Option<T>
where
    T: WidthSliceable,
{
    type Output = T::Output;
    fn slice_width<R>(&self, range: R) -> Option<Self::Output>
    where
        R: RangeBounds<usize>,
    {
        match self {
            Some(t) => t.slice_width(range),
            None => None,
        }
    }
}
