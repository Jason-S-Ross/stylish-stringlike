use super::StyledGrapheme;
use ansi_term::{ANSIString, Style};
use std::ops::Deref;
use unicode_segmentation::UnicodeSegmentation;
pub trait Graphemes<'a, T: Clone> {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a, T>> + 'a>;
}

impl<'a> Graphemes<'a, Style> for ANSIString<'a> {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a, Style>> + 'a> {
        Box::new(
            self.deref().graphemes(true).map(move |grapheme| {
                StyledGrapheme::<'a, Style>::borrowed(self.style_ref(), grapheme)
            }),
        )
    }
}

impl<'a> Graphemes<'a, Style> for Vec<ANSIString<'a>> {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a, Style>> + 'a> {
        Box::new(self.iter().flat_map(move |s| {
            let style = s.style_ref();
            s.deref()
                .graphemes(true)
                .map(move |grapheme| StyledGrapheme::<'a, Style>::borrowed(style, grapheme))
        }))
    }
}
