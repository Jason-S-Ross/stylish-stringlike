mod repeat;
mod text;
mod text_widget;

use crate::text::Text;
use crate::text::{Span, Spans};
use crate::text_widget::{HBox, TextWidget, Truncatable, TruncationStyle};
use ansi_term::Color;

fn main() {
    let texts = vec![
        Color::Red.paint("Hello, World! "),
        Color::Blue.paint("Here lies Beavis, "),
        Color::Green.paint("He never "),
        Color::Cyan.paint("scored"),
    ];
    let groups: Vec<Span> = texts.iter().map(|x| x.into()).collect();
    let text: Spans = groups.iter().flat_map(|x| x.graphemes()).collect();
    let ellipsis = Color::Blue.paint("…");
    let ellipsis_span: Span = (&ellipsis).into();
    let widgets: Vec<TextWidget<_>> = groups
        .iter()
        .map(|x| TextWidget::new(x, TruncationStyle::Left, &ellipsis_span))
        .collect();
    let widget_refs: Vec<&TextWidget<_>> = widgets.iter().collect();
    let hbox = HBox::new(&widget_refs);
    for width in 0..30 {
        println!(
            "span truncated: {}",
            text.truncate_left(width, &ellipsis_span).collect::<Spans>()
        );
        println!(
            "span truncated: {}",
            text.truncate_right(width, &ellipsis_span)
                .collect::<Spans>()
        );
        println!(
            "span truncated: {}",
            text.truncate_outer(width, &ellipsis_span)
                .collect::<Spans>()
        );
        println!(
            "span truncated: {}",
            text.truncate_inner(width, &ellipsis_span)
                .collect::<Spans>()
        );
        println!(
            "widget:         {}",
            hbox.truncate(width).collect::<Spans>()
        )
    }
}
