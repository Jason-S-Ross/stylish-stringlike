mod segmentation;
mod span;
mod stringlike;
use ansi_term::Color;
use segmentation::UnicodeSegmentation;
use span::Span;

fn main() {
    let spans = [Color::Red.paint("Hello "), Color::Blue.paint("World")];
    let span: Span = spans.iter().collect();

    println!("Spans: {}", span);
    for word in span.split_word_bound_indices() {
        println!("{:?}", word);
    }
}
