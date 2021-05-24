use crate::text::{HasWidth, Span, Width, WidthSliceable};
use std::borrow::Cow;
use std::ops::{Bound, RangeBounds};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub(crate) struct Repeat<'a, T: Clone> {
    content: Cow<'a, str>,
    style: Cow<'a, T>,
}

impl<'a, T: Clone> Repeat<'a, T> {
    pub(crate) fn new(style: Cow<'a, T>, content: Cow<'a, str>) -> Repeat<'a, T> {
        Repeat { content, style }
    }
}

impl<'a, T: Clone> HasWidth for Repeat<'a, T> {
    fn width(&self) -> Width {
        Width::Unbounded
    }
}

impl<'a, T: Clone> WidthSliceable<'a> for Repeat<'a, T> {
    type Output = Span<'a, T>;
    fn slice_width(&'a self, range: (Bound<usize>, Bound<usize>)) -> Option<Self::Output> {
        let normalized_range = match range {
            (Bound::Excluded(s), Bound::Excluded(e)) => (Bound::Unbounded, Bound::Excluded(e - s)),
            (Bound::Excluded(s), Bound::Included(e)) => (Bound::Unbounded, Bound::Included(e - s)),
            (Bound::Included(s), Bound::Excluded(e)) => (Bound::Unbounded, Bound::Excluded(e - s)),
            (Bound::Included(s), Bound::Included(e)) => (Bound::Unbounded, Bound::Excluded(e - s)),
            (Bound::Unbounded, Bound::Excluded(e)) => (Bound::Unbounded, Bound::Excluded(e)),
            (Bound::Unbounded, Bound::Included(e)) => (Bound::Unbounded, Bound::Included(e)),
            _ => return None,
        };
        let mut content = String::new();
        let mut current_width = 0;
        while normalized_range.contains(&current_width) {
            for grapheme in self.content.graphemes(true) {
                current_width += grapheme.width();
                if !normalized_range.contains(&current_width) {
                    break;
                }
                content += grapheme;
            }
        }
        Some(Span::new(Cow::Borrowed(&self.style), Cow::Owned(content)))
    }
}
