stylish_stringlike
==================
 
This crate provides a string-like API for styled text objects,
and widgets for displaying those styled text objects specifically
oriented towards terminal output.

## usage

Add this to you `Cargo.toml`:

```toml
[dependencies.stylish-stringlike]
git = "https://github.com/Jason-S-Ross/stylish-stringlike.git"
version = "0.1.0"
```

### Example

``` rust
use std::borrow::Cow;
use stylish_stringlike::text::{Joinable, Paintable, Replaceable, Sliceable, Span, Spans, Tag};
use stylish_stringlike::widget::{HBox, TextWidget, TruncationStyle};

let italic = Tag::new("<i>", "</i>");
let bold = Tag::new("<b>", "</b>");
let underline = Tag::new("<u>", "</u>");

let foo: Span<Tag> = Span::new(Cow::Borrowed(&italic), Cow::Borrowed("foo"));
let bar: Span<Tag> = Span::new(Cow::Borrowed(&bold), Cow::Borrowed("bar"));
let foobar = foo.join(&bar);
assert_eq!(format!("{}", foobar), "<i>foo</i><b>bar</b>");
let foobaz = foobar.replace("bar", "baz");
assert_eq!(format!("{}", foobaz), "<i>foo</i><b>baz</b>");
let mut buz: Spans<Tag> = Default::default();
buz = buz.join(&Span::new(
    Cow::Borrowed(&underline),
    Cow::Owned(String::from("buz")),
));
let foobuz = foobar.replace("bar", &buz);
assert_eq!(format!("{}", foobuz), "<i>foo</i><u>buz</u>");
let foob = foobar.slice(..4).unwrap();
assert_eq!(format!("{}", foob), "<i>foo</i><b>b</b>");
fn make_spans(style: &Tag, text: &str) -> Spans<Tag> {
    let mut spans: Spans<Tag> = Default::default();
    let span: Span<Tag> = Span::new(Cow::Borrowed(style), Cow::Borrowed(text));
    spans = spans.join(&span);
    spans
}
let truncation = TruncationStyle::Inner(Some(Span::new(
    Cow::Borrowed(&underline),
    Cow::Owned(String::from("…")),
)));
let first_spans = make_spans(&italic, "abcdefg");
let second_spans = make_spans(&bold, "12345678");
let first_segment = TextWidget::new(&first_spans, &truncation);

let second_segment = TextWidget::new(&second_spans, &truncation);
let mut hbox: HBox<Spans<Tag>> = Default::default();
hbox.push(&first_segment);
hbox.push(&second_segment);
assert_eq!(
    format!("{}", hbox.truncate(10)),
    "<i>ab</i><u>…</u><i>fg</i><b>12</b><u>…</u><b>78</b>"
);

```
