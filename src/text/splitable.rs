use super::{RawText, Sliceable};
use std::iter::once;

pub trait Splitable<'a, T> {
    /// The type corresponding to matched delimiters
    type Delim;
    /// The type corresponding to segments bounded by delimiters
    type Segment;
    #[allow(clippy::type_complexity)]
    fn split(
        &'a self,
        pattern: T,
    ) -> Box<dyn Iterator<Item = (Option<Self::Segment>, Option<Self::Delim>)> + 'a>;
}

impl<'a, T> Splitable<'a, &'a str> for T
where
    T: Sliceable<'a, Output = T, Index = usize> + RawText,
{
    type Delim = Self;
    type Segment = Self;
    #[allow(clippy::type_complexity)]
    fn split(
        &'a self,
        pattern: &'a str,
    ) -> Box<dyn Iterator<Item = (Option<Self::Segment>, Option<Self::Delim>)> + 'a> {
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
                            Some((None, delim))
                        } else {
                            Some((self.slice(*last_end..start), delim))
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
                            Some((self.slice(*last_end..), None))
                        }
                    }
                }),
        )
    }
}
