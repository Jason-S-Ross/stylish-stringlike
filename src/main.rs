mod segmentation;
mod span;
mod stringlike;
mod span_widget;
use ansi_term::Color;
use segmentation::UnicodeSegmentation;
use span::Span;
use span_widget::SpanWidget;

fn main() {
    let spans = [
        Color::Red.paint("🍽🍠👅🍑⌣🍴"),
        Color::Blue.paint("☹☺⌢😢")
    ];
    let span: Span = spans.iter().collect();

    println!("Spans: {}", span);
    for word in span.split_word_bound_indices() {
        println!("{:?}", word);
    }
    let ellipsis_colors = Color::Green.paint("……");
    let ellipsis: Span = (&ellipsis_colors).into();
    let span_widget = SpanWidget::new(&span, &ellipsis, None);
    for grapheme in span[..].iter() {
        println!("{:?}", grapheme);
    }
    println!("span_widget:  {}", span_widget);
    for width in 1..=10 {
        let shrunk = span_widget.shrink(width);
        println!("Shrunk {:03}: {}", width, shrunk)
    }
}
