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
//!
//! ## Usage
//!
//! ```
//! use stylish_stringlike::text::{Span, Spans, Painter, Joinable, Replaceable, Sliceable};
//! use stylish_stringlike::widget::{TruncationStyle, HBox, TextWidget};
//! use std::borrow::Cow;
//!
//! #[derive(Clone, Default, PartialEq)]
//! struct MyMarkup {
//!     tag: String,
//! }
//! impl Painter for MyMarkup {
//!     fn paint(&self, target: &str) -> String {
//!         [
//!             format!("<{}>", self.tag).as_str(),
//!             target,
//!             format!("</{}>", self.tag).as_str()
//!         ].iter().map(|x| *x).collect()
//!     }
//! }
//! let italic = MyMarkup {
//!     tag: String::from("i"),
//! };
//! let bold = MyMarkup {
//!     tag: String::from("b"),
//! };
//! let underline = MyMarkup {
//!     tag: String::from("u"),
//! };
//! let foo: Span<MyMarkup> = Span::new(Cow::Borrowed(&italic), Cow::Owned(String::from("foo")));
//! let bar: Span<MyMarkup> = Span::new(Cow::Borrowed(&bold), Cow::Owned(String::from("bar")));
//! let foobar = foo.join(&bar);
//! assert_eq!(format!("{}", foobar), "<i>foo</i><b>bar</b>");
//! let foobaz = foobar.replace("bar", "baz");
//! assert_eq!(format!("{}", foobaz), "<i>foo</i><b>baz</b>");
//! let mut buz: Spans<MyMarkup> = Default::default();
//! buz = buz.join(&Span::new(Cow::Borrowed(&underline), Cow::Owned(String::from("buz"))));
//! let foobuz = foobar.replace("bar", &buz);
//! assert_eq!(format!("{}", foobuz), "<i>foo</i><u>buz</u>");
//! let foob = foobar.slice(..4).unwrap();
//! assert_eq!(format!("{}", foob), "<i>foo</i><b>b</b>");
//! fn make_spans(style: &MyMarkup, text: &str) -> Spans<MyMarkup> {
//!     let mut spans: Spans<MyMarkup> = Default::default();
//!     let span: Span<MyMarkup> = Span::new(Cow::Borrowed(style), Cow::Borrowed(text));
//!     spans = spans.join(&span);
//!     spans
//! }
//! let truncation = TruncationStyle::Inner(Some(Span::new(Cow::Borrowed(&underline), Cow::Owned(String::from("…")))));
//! let first_spans = make_spans(&italic, "abcdefg");
//! let second_spans = make_spans(&bold, "12345678");
//! let first_segment = TextWidget::new(
//!     &first_spans,
//!     &truncation,
//! );
//!     
//! let second_segment = TextWidget::new(
//!     &second_spans,
//!     &truncation,
//! );
//! let mut hbox: HBox = Default::default();
//! hbox.push(&first_segment);
//! hbox.push(&second_segment);
//! assert_eq!(format!("{}", hbox.truncate(10)), "<i>ab</i><u>…</u><i>fg</i><b>12</b><u>…</u><b>78</b>");
//! ```
pub mod text;
pub mod widget;

#[cfg(test)]
mod test {
    use super::*;
    use ansi_term::{ANSIString, Color, Style};
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
