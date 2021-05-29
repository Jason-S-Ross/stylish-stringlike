stylish_stringlike
==================
 
This crate provides a string-like API for styled text objects,
and widgets for displaying those styled text objects specifically
oriented towards terminal output.

## Usage

Add this to you `Cargo.toml`:

```toml
[dependencies]
stylish-stringlike = "0.2.0"
```

### Example

``` rust
use std::borrow::Cow;
use stylish_stringlike::text::{Joinable, Paintable, 
Pushable, Replaceable, Sliceable, Span, Spans, Tag};
use stylish_stringlike::widget::{Fitable, HBox, 
TextWidget, TruncationStyle};

let italic = Tag::new("<i>", "</i>");
let bold = Tag::new("<b>", "</b>");
let underline = Tag::new("<u>", "</u>");

let foo: Span<Tag> = Span::new(
    Cow::Borrowed(&italic), Cow::Borrowed("foo"));
let bar: Span<Tag> = Span::new(
    Cow::Borrowed(&bold), Cow::Borrowed("bar"));

// Spans of different styles can be joined together.
let foobar = foo.join(&bar);
assert_eq!(format!("{}", foobar), "<i>foo</i><b>bar</b>");

// Perform literal string replacement with the `replace` 
// method.
let foobaz = foobar.replace("bar", "baz");
assert_eq!(format!("{}", foobaz), "<i>foo</i><b>baz</b>");

let mut buz: Spans<Tag> = Default::default();
buz.push(&Span::new(
    Cow::Borrowed(&underline), Cow::Borrowed("buz")));

// Replace text with styled text objects instead of string 
// literals.
let foobuz = foobar.replace("bar", &buz);
assert_eq!(format!("{}", foobuz), "<i>foo</i><u>buz</u>");

// Use the `slice` method to slice on bytes.
let foob = foobar.slice(..4).unwrap();
assert_eq!(format!("{}", foob), "<i>foo</i><b>b</b>");

// Use the `HBox` widget to truncate multiple spans of text 
// to fit in a desired width.
fn make_spans(style: &Tag, text: &str) -> Spans<Tag> {
    let mut spans: Spans<Tag> = Default::default();
    let span: Span<Tag> = Span::new(
        Cow::Borrowed(style), Cow::Borrowed(text));
    spans = spans.join(&span);
    spans
}
let truncation = TruncationStyle::Inner(Some(Span::new(
    Cow::Borrowed(&underline),
    Cow::Borrowed("…"),
)));
let spans = vec![
    make_spans(&italic, "abcdefg"), 
    make_spans(&bold, "12345678"),
];
let hbox = spans
    .iter()
    .map(|s| {
        let b: Box<dyn Fitable<_>> =
            Box::new(TextWidget::<Spans<_>, 
                TruncationStyle<_>>::new(
                    Cow::Borrowed(s),
                    Cow::Borrowed(&truncation),
                )
            );
        b
    })
    .collect::<HBox<_>>();
assert_eq!(
    format!("{}", hbox.truncate(10)),
    "<i>ab</i><u>…</u><i>fg</i><b>12</b><u>…</u><b>78</b>"
);


```
