use super::{RawText, Sliceable};
use std::iter::once;

#[derive(Clone, Debug, Eq, PartialEq)]
/// A segment of text split on a delimiter.
/// The delimiter and the segment are both included because
/// the delimiter may have a style applied to it.
pub struct Split<T, U> {
    pub delim: Option<T>,
    pub segment: Option<U>,
}

/// Text objects that can be split on a delimiter or pattern
pub trait Splitable<'a, T> {
    // TODO: Rename this split
    /// Split a text object on the given pattern
    fn split_style(&'a self, pattern: T) -> Box<dyn Iterator<Item = Split<Self, Self>> + 'a>
    where
        Self: Sized;
}

impl<'a, T> Splitable<'a, &'a str> for T
where
    T: Sliceable<'a> + RawText,
{
    #[allow(clippy::type_complexity)]
    fn split_style(&'a self, pattern: &'a str) -> Box<dyn Iterator<Item = Split<Self, Self>> + 'a> {
        Box::new(
            self.raw_ref()
                .match_indices(pattern)
                // This is a silly hack to flag when we get to the last element in the list.
                // Items in the list will be Some.
                .map(Some)
                // The last item will be None.
                .chain(once(None))
                .scan(0, move |last_end, item| {
                    if let Some((start, pat)) = item {
                        let end = start + pat.len();
                        let delim = self.slice(start..end);
                        let res = if start == 0 {
                            // String starts with delimiter
                            Some(Split {
                                segment: None,
                                delim,
                            })
                        } else {
                            Some(Split {
                                segment: self.slice(*last_end..start),
                                delim,
                            })
                        };
                        *last_end = end;
                        res
                    } else {
                        // This is the last item.
                        if *last_end == self.raw().len() {
                            // After consuming the last match, we are at the end of the string
                            None
                        } else {
                            // After consuming the last match, we still have some string yet
                            Some(Split {
                                segment: self.slice(*last_end..),
                                delim: None,
                            })
                        }
                    }
                }),
        )
    }
}
