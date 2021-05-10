mod text;
mod text_widget;

use crate::text::Text;
use crate::text::{Span, Spans};
use crate::text_widget::Truncatable;
use ansi_term::{Color, ANSIStrings};

fn main() {
    let words = Color::Red.paint("Hello, World!");
    let more_words = Color::Blue.paint("Here lies Beavis");
    let groups: Vec<Span> = vec![(&words).into(), (&more_words).into()];
    let text: Spans = groups.iter().flat_map(|x| x.graphemes()).collect();
    let ellipsis = Color::Blue.paint("…");
    let ellipsis_span: Span = (&ellipsis).into();
    for width in 0..30 {
        println!("span truncated: {}", text.truncate_left(width, &ellipsis_span).collect::<Spans>());
        println!("span truncated: {}", text.truncate_right(width, &ellipsis_span).collect::<Spans>());
        println!("span truncated: {}", text.truncate_outer(width, &ellipsis_span).collect::<Spans>());
        println!("span truncated: {}", text.truncate_inner(width, &ellipsis_span).collect::<Spans>());

    }
}
