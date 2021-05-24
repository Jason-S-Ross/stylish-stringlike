mod text;
mod widget;
pub(crate) use text::*;
pub(crate) use widget::*;

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::{Repeat, Span, Spans};
    use ansi_term::{ANSIString, ANSIStrings, Color, Style};
    use std::borrow::Cow;
    use std::path::Path;
    use truncatable::{Truncateable, TruncationStyle};
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
