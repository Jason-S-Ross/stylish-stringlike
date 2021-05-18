mod text;
mod widget;
pub use text::*;
pub use widget::*;

fn main() {}
#[cfg(test)]
mod test {
    use super::*;
    use ansi_term::Color;
    use std::path::Path;
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
        let spans: Spans<_> = (&texts).into();
        let split = spans.split_style("::").collect::<Vec<_>>();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans<_> = (&ellipsis).into();
        let widget_container = split
            .iter()
            .filter_map(|Split { segment, delim }| match (segment, delim) {
                (Some(segment), Some(delim)) => Some(vec![
                    TextWidget::new(segment, TruncationStyle::Inner, &ellipsis_span),
                    TextWidget::new(delim, TruncationStyle::Inner, &ellipsis_span),
                ]),
                (Some(segment), None) => Some(vec![TextWidget::new(
                    segment,
                    TruncationStyle::Inner,
                    &ellipsis_span,
                )]),
                (None, Some(delim)) => Some(vec![TextWidget::new(
                    delim,
                    TruncationStyle::Inner,
                    &ellipsis_span,
                )]),
                (None, None) => None,
            })
            .flatten()
            .collect::<Vec<_>>();
        let widgets = widget_container.iter().collect::<Vec<_>>();
        let hbox = HBox::new(&widgets);
        println!("{}", hbox.truncate(20).collect::<Spans<_>>());
    }
}
