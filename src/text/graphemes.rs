use super::StyledGrapheme;
use ansi_term::ANSIString;
use std::ops::Deref;
use unicode_segmentation::UnicodeSegmentation;
pub trait Graphemes<'a> {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a>;
}

impl<'a> Graphemes<'a> for ANSIString<'a> {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        Box::new(
            self.deref()
                .graphemes(true)
                .map(move |grapheme| StyledGrapheme::borrowed(self.style_ref(), grapheme)),
        )
    }
}

impl<'a> Graphemes<'a> for Vec<ANSIString<'a>> {
    fn graphemes(&'a self) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        Box::new(self.iter().flat_map(move |s| {
            let style = s.style_ref();
            s.deref()
                .graphemes(true)
                .map(move |grapheme| StyledGrapheme::borrowed(style, grapheme))
        }))
    }
}
