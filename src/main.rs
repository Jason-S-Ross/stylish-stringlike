mod repeat;
mod text;
mod text_widget;

use crate::repeat::Repeat;
use crate::text::{Graphemes, Spans};
use crate::text_widget::{HBox, TextWidget, TruncationStyle};
use ansi_term::Color;

fn main() {
    let texts = vec![
        Color::Red.paint("Here is fo"),
        Color::Blue.paint("o some foo text f"),
        Color::Green.paint("oo. \nThis should look like"),
        Color::Red.paint("Here is some ba"),
        Color::Blue.paint("r some bar text b"),
        Color::Green.paint("ar"),
    ];
    let text: Spans = (&texts).into();
    let text = text.replace("foo", "bar");
    for span in text.spans() {
        println!("{}", span);
    }
    let texts: Vec<Spans> = texts.iter().map(|x| x.into()).collect();
    let ellipsis = Color::Blue.paint("…");
    let ellipsis_span: Spans = (&ellipsis).into();
    let empty = Color::Black.paint("");
    let empty_span: Spans = (&empty).into();
    let equals = Color::Yellow.paint("🏢");
    let g = (&equals).graphemes().next().unwrap();
    let repeat_widget = Repeat::new(g);
    let repeat_text_widget = TextWidget::new(&repeat_widget, TruncationStyle::Left, &empty_span);
    let mut widgets: Vec<TextWidget> = texts
        .iter()
        .map(|x| TextWidget::new(x, TruncationStyle::Left, &ellipsis_span))
        .collect();
    widgets.insert(0, repeat_text_widget);
    let widget_refs: Vec<&TextWidget> = widgets.iter().collect();
    let hbox = HBox::new(&widget_refs);
    println!("Result: {}", text);
    for width in 100..101 {
        println!(
            "widget:         {}",
            hbox.truncate(width).collect::<Spans>()
        )
    }
}
