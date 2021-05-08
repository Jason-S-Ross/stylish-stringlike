use crate::span::{Span, StyledGrapheme};
use std::fmt::Display;
use std::fmt;


pub struct SpanWidget<'a> {
    span: &'a Span<'a>,
    ellipsis: &'a Span<'a>,
    min_size: Option<usize>,
}

pub struct ShrunkSpanWidget<'a> {
    span: &'a SpanWidget<'a>,
    width: usize
}

impl<'a> fmt::Display for SpanWidget<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.span.fmt(f)
    }
}

impl<'a> fmt::Display for ShrunkSpanWidget<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.span.shrink_to(self.width, f)
    }
}

impl<'a> SpanWidget<'a> {
    pub fn new(span: &'a Span, ellipsis: &'a Span, min_size: Option<usize>) -> Self {
        SpanWidget{ span, ellipsis, min_size }
    }
    fn width(&self) -> usize {
        self.span.width()
    }
    pub fn shrink(&self, width: usize) -> ShrunkSpanWidget {
        ShrunkSpanWidget { span: self, width }
    }
    fn truncate_slice_left_width<'b>(&'b self, width: usize) -> &'b [StyledGrapheme<'a>] {
        let mut slice_width = 0;
        for grapheme in self.span[..].iter() {
            slice_width += grapheme.grapheme_width();
            if slice_width >= width {
                break
            }
        }
        &self.span[..slice_width]
    }
    fn shrunk_left<'b>(&'b self, width: usize) -> Span<'b> {
        if self.width() <= width {
            (*self.span).clone()
        } else {
            let graphemes = {
                let mut graphemes: Vec<StyledGrapheme> = vec![];
                match (width, self.ellipsis.width(), self.width()) {
                    (w, ew, sw) if (w <= ew) => {
                        graphemes.extend_from_slice(&self.span[sw - w..])
                    }
                    (w, ew, sw) => {
                        graphemes.extend_from_slice(&self.ellipsis[..]);
                        graphemes.extend_from_slice(&self.span[sw - w + ew..]);
                    }
                }
                graphemes
            };
            Span::new(graphemes)
        }
    }
    fn shrunk_right<'b>(&'b self, width: usize) -> Span<'b> {
        if self.width() <= width {
            (*self.span).clone()
        } else {
            let graphemes = {
                let mut graphemes: Vec<StyledGrapheme> = vec![];
                match (width, self.ellipsis.width(), self.width()) {
                    (w, ew, sw) if (w <= ew) => {
                        graphemes.extend_from_slice(&self.span[..w])
                    }
                    (w, ew, sw) => {
                        graphemes.extend_from_slice(&self.truncate_slice_left_width(w - ew));
                        graphemes.extend_from_slice(&self.ellipsis[..]);
                    }
                }
                graphemes
            };
            Span::new(graphemes)
        }
    }
    fn shrunk<'b>(&'b self, width: usize) -> Span<'b> {
        if self.width() <= width {
            (*self.span).clone()
        } else {
            let graphemes = {
                let mut graphemes: Vec<StyledGrapheme> = vec![];
                match (width, self.ellipsis.width(), self.width()) {
                    (w, ew, sw) if (w < ew && w > 1) || (w < ew + 2) => {
                        let odd = (w % 2);
                        let first_bound = w / 2 + odd;
                        let second_bound = sw - (first_bound - odd);
                        graphemes.extend_from_slice(&self.span[..first_bound]);
                        graphemes.extend_from_slice(&self.span[second_bound..]);
                    }
                    (w, ew, sw) => {
                        let odd = ((w - ew) % 2);
                        let first_bound = (w - ew) / 2 + odd;
                        let second_bound = sw - (first_bound - odd);
                        graphemes.extend_from_slice(&self.span[..first_bound]);
                        graphemes.extend_from_slice(&self.ellipsis[..]);
                        graphemes.extend_from_slice(&self.span[second_bound..]);
                    }
                }
                graphemes
            };
            Span::new(graphemes)
        }
    }
    fn shrink_to(&self, width: usize, f: &mut fmt::Formatter) -> fmt::Result {
        if self.width() <= width {
            self.fmt(f)
        } else {
            let graphemes = {
                let mut graphemes: Vec<StyledGrapheme> = vec![];
                match (width, self.ellipsis.width(), self.width()) {
                    (w, ew, sw) if (w < ew && w > 1) || (w < ew + 2) => {
                        let odd = (w % 2);
                        let first_bound = w / 2 + odd;
                        let second_bound = sw - (first_bound - odd);
                        graphemes.extend_from_slice(&self.span[..first_bound]);
                        graphemes.extend_from_slice(&self.span[second_bound..]);
                    }
                    (w, ew, sw) => {
                        let odd = ((w - ew) % 2);
                        let first_bound = (w - ew) / 2 + odd;
                        let second_bound = sw - (first_bound - odd);
                        graphemes.extend_from_slice(&self.span[..first_bound]);
                        graphemes.extend_from_slice(&self.ellipsis[..]);
                        graphemes.extend_from_slice(&self.span[second_bound..]);
                    }
                }
                graphemes
            };
            let new_span: Span = graphemes.iter().collect();
            new_span.fmt(f)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ansi_term::{Color, ANSIStrings};
    use crate::span::Span;
    use unicode_width::UnicodeWidthStr;
    #[test]
    fn check_empty_ellipsis() {
        let spans = [
            Color::Red.paint("🍽🍠👅🍑⌣🍴"),
            Color::Blue.paint("ab̖̀̑☹☺⌢😢")
        ];
        eprintln!("span1:   {}", spans[1]);
        eprintln!("spans:   {}", ANSIStrings(&spans));
        let span: Span = spans.iter().collect();
        let ellipsis_term = Color::Green.paint("...");
        let ellipsis: Span = (&ellipsis_term).into();
        let span_widget = SpanWidget::new(&span, &ellipsis, None);
        for width in 1..=12 {
            let shrunk = span_widget.shrunk(width);
            eprintln!("Shrunk:    {}", shrunk);
            assert_eq!(shrunk.width(), width);
        }
        for width in 1..=12 {
            let shrunk = span_widget.shrunk_left(width);
            eprintln!("Shrunk:    {}", shrunk);
            assert_eq!(shrunk.width(), width);
        }
        for width in 1..=12 {
            let shrunk = span_widget.shrunk_right(width);
            eprintln!("width:       {}", shrunk[..].iter().map(|x| x.grapheme_width()).sum::<usize>());
            eprintln!("Shrunk:    {}", shrunk);
        }

    }
}
