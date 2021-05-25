//! This is a libary for creating styled spans of text. The style can be
//! something like an ANSI terminal color/format, or it could be some
//! markup like enclosing text in html tags.
//!
//!
//! ## Structure
//! This crate is subdivided into two modules: [`text`] and [`widget`].
//!
//! [`text`] provides string-like functionality for styled pieces of text.
//! Methods such as replacing, slicing, and splitting are supported while
//! respecting existing style delimiters.
//!
//! [`widget`] provides functionality for displaying text objects in useful ways,
//! such as truncation with a symbol, or repeating a sequence.
pub mod text;
pub mod widget;

#[cfg(test)]
mod test {
    use super::*;
    use ansi_term::{ANSIString, ANSIStrings, Color, Style};
    use std::borrow::Cow;
    use std::path::Path;
    use text::*;
    use widget::*;
    fn make_spans(style: &Style, text: &str) -> Spans<Style> {
        let ansistring: ANSIString = Style::paint(*style, text);
        let span: Span<Style> = ansistring.into();
        let mut spans: Spans<Style> = Default::default();
        spans.push_span(&span);
        spans
    }
    #[test]
    fn split_path() {
        let texts = vec![
            Color::Black.paint("::"),
            Color::Red.paint("SomeExtremelyLong"),
            Color::Blue.paint("::"),
            Color::Green.paint("RandomAndPoorlyNamed"),
            Color::Cyan.paint("::"),
            Color::White.paint("Path"),
            Color::Yellow.paint("::"),
        ];
        let spans: Spans<_> = texts.iter().map(Span::<Style>::from).collect();
        let split = spans.split_style("::").collect::<Vec<_>>();
        let ellipsis_string = Color::Blue.paint("…");
        let ellipsis_span = make_spans(&Color::Blue.normal(), "…");
        let mut hbox = HBox::new();
        let truncation = TruncationStyle::Inner(Some(ellipsis_span));
        let widget_container = split
            .iter()
            .filter_map(|Split { segment, delim }| match (segment, delim) {
                (Some(segment), Some(delim)) => Some(vec![
                    TextWidget::new(segment, &truncation),
                    TextWidget::new(delim, &truncation),
                ]),
                (Some(segment), None) => Some(vec![TextWidget::new(segment, &truncation)]),
                (None, Some(delim)) => Some(vec![TextWidget::new(delim, &truncation)]),
                (None, None) => None,
            })
            .flatten()
            .collect::<Vec<_>>();
        let mut hbox = HBox::new();
        for widget in &widget_container {
            hbox.push(widget);
        }
        let actual = hbox.truncate(20);
        let expected = format!(
            "{}{}{}{}{}{}{}{}{}{}{}",
            Color::Black.paint("::"),
            Color::Red.paint("So"),
            ellipsis_string,
            Color::Red.paint("g"),
            Color::Blue.paint("::"),
            Color::Green.paint("Ra"),
            ellipsis_string,
            Color::Green.paint("d"),
            Color::Cyan.paint("::"),
            Color::White.paint("Path"),
            Color::Yellow.paint("::"),
        );
        assert_eq!(expected, actual);
    }
}
