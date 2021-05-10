mod repeat;
mod text;
mod text_widget;

use crate::repeat::Repeat;
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
    // let repeat =
    let groups: Vec<Span> = texts.iter().map(|x| x.into()).collect();
    let text: Spans = groups.iter().flat_map(|x| x.graphemes()).collect();
    let repeat_ansi = Color::Yellow.paint("=");
    let repeat_span: Span = (&repeat_ansi).into();
    let g = repeat_span.graphemes().next().unwrap();
    let ellipsis = Color::Blue.paint("…");
    let ellipsis_span: Span = (&ellipsis).into();
    let repeat_widget = Repeat::new(g);
    let repeat_text_widget = TextWidget::new(&repeat_widget, TruncationStyle::Left, &ellipsis_span);
    let mut widgets: Vec<TextWidget<_>> = groups
        .iter()
        .map(|x| TextWidget::new(x, TruncationStyle::Left, &ellipsis_span))
        .collect();
    widgets.insert(0, repeat_text_widget);
    let widget_refs: Vec<&TextWidget<_>> = widgets.iter().collect();
    let hbox = HBox::new(&widget_refs);
    for width in 0..50 {
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
