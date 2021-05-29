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
//! ```rust
//! use std::borrow::Cow;
//! use stylish_stringlike::text::{Joinable, Paintable, Pushable, Replaceable, Sliceable, Span,
//!     Spans, Tag};
//! use stylish_stringlike::widget::{Fitable, HBox, TextWidget, TruncationStyle};
//!
//! let italic = Tag::new("<i>", "</i>");
//! let bold = Tag::new("<b>", "</b>");
//! let underline = Tag::new("<u>", "</u>");
//!
//! let foo: Span<Tag> = Span::new(Cow::Borrowed(&italic), Cow::Borrowed("foo"));
//! let bar: Span<Tag> = Span::new(Cow::Borrowed(&bold), Cow::Borrowed("bar"));
//!
//! // Spans of different styles can be joined together.
//! let foobar = foo.join(&bar);
//! assert_eq!(format!("{}", foobar), "<i>foo</i><b>bar</b>");
//!
//! // Perform literal string replacement with the `replace` method.
//! let foobaz = foobar.replace("bar", "baz");
//! assert_eq!(format!("{}", foobaz), "<i>foo</i><b>baz</b>");
//!
//! let mut buz: Spans<Tag> = Default::default();
//! buz.push(&Span::new(Cow::Borrowed(&underline), Cow::Borrowed("buz")));
//!
//! // Replace text with styled text objects instead of string literals.
//! let foobuz = foobar.replace("bar", &buz);
//! assert_eq!(format!("{}", foobuz), "<i>foo</i><u>buz</u>");
//!
//! // Use the `slice` method to slice on bytes.
//! let foob = foobar.slice(..4).unwrap();
//! assert_eq!(format!("{}", foob), "<i>foo</i><b>b</b>");
//!
//! // Use the `HBox` widget to truncate multiple spans of text to fit in a desired width.
//! fn make_spans(style: &Tag, text: &str) -> Spans<Tag> {
//!     let mut spans: Spans<Tag> = Default::default();
//!     let span: Span<Tag> = Span::new(Cow::Borrowed(style), Cow::Borrowed(text));
//!     spans = spans.join(&span);
//!     spans
//! }
//! let truncation = TruncationStyle::Inner(Some(Span::new(
//!     Cow::Borrowed(&underline),
//!     Cow::Borrowed("…"),
//! )));
//! let spans = vec![make_spans(&italic, "abcdefg"), make_spans(&bold, "12345678")];
//! let hbox = spans
//!     .iter()
//!     .map(|s| {
//!         let b: Box<dyn Fitable<_>> =
//!             Box::new(TextWidget::<Spans<_>, TruncationStyle<_>>::new(
//!                 Cow::Borrowed(s),
//!                 Cow::Borrowed(&truncation),
//!             ));
//!         b
//!     })
//!     .collect::<HBox<_>>();
//! assert_eq!(
//!     format!("{}", hbox.truncate(10)),
//!     "<i>ab</i><u>…</u><i>fg</i><b>12</b><u>…</u><b>78</b>"
//! );
//! ```
pub mod text;
pub mod widget;

#[cfg(test)]
mod test {
    use super::*;
    use ansi_term::{ANSIString, ANSIStrings, Color, Style};
    use std::borrow::Cow;
    use text::*;
    use widget::*;
    fn make_spans(style: &Style, text: &str) -> Spans<Style> {
        let ansistring: ANSIString = Style::paint(*style, text);
        let span: Span<Style> = ansistring.into();
        let mut spans: Spans<Style> = Default::default();
        spans.push(&span);
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
        let ellipsis_string = Color::Blue.paint("…");
        let ellipsis_span = make_spans(&Color::Blue.normal(), "…");
        let truncation = TruncationStyle::Inner(ellipsis_span.clone());

        let actual = spans
            .split("::")
            .map(|Split { segment, delim }| vec![segment, delim])
            .flatten()
            .flatten()
            .map(|s| {
                let foo: Box<dyn Fitable<_>> =
                    Box::new(TextWidget::<Spans<_>, TruncationStyle<_>>::new(
                        Cow::Owned(s),
                        Cow::Borrowed(&truncation),
                    ));
                foo
            })
            .collect::<HBox<_>>()
            .truncate(20)
            .to_string();
        let expected = format!(
            "{}",
            ANSIStrings(&[
                Color::Black.paint("::"),
                Color::Red.paint("So"),
                ellipsis_string.clone(),
                Color::Red.paint("g"),
                Color::Blue.paint("::"),
                Color::Green.paint("Ra"),
                ellipsis_string,
                Color::Green.paint("d"),
                Color::Cyan.paint("::"),
                Color::White.paint("Path"),
                Color::Yellow.paint("::"),
            ])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn split_path_2() {
        let path = Color::Blue.paint("some//path//with//segments");
        let span: Span<Style> = path.clone().into();
        let spans = {
            let mut spans: Spans<Style> = Default::default();
            spans.push(&span);
            spans
        };
        let truncation = TruncationStyle::Inner(Some(make_spans(&Color::Blue.normal(), "……")));

        let actual = spans
            .split("::")
            .map(|Split { segment, delim }| vec![segment, delim])
            .flatten()
            .flatten()
            .map(|s| {
                let foo: Box<dyn Fitable<_>> =
                    Box::new(TextWidget::<Spans<Style>, TruncationStyle<_>>::new(
                        Cow::Owned(s),
                        Cow::Borrowed(&truncation),
                    ));
                foo
            })
            .collect::<HBox<_>>()
            .truncate(50)
            .to_string();

        let expected = format!("{}", path);
        assert_eq!(expected, actual);
    }
}
