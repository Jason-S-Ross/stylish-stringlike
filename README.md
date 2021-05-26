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
use stylish_stringlike::text::{Span, Spans, Painter, Joinable, Replaceable, Sliceable};
use stylish_stringlike::widget::{TruncationStyle, HBox, TextWidget};
use std::borrow::Cow;

#[derive(Clone, Default, PartialEq)]
struct MyMarkup {
    tag: String,
}
impl Painter for MyMarkup {
    fn paint(&self, target: &str) -> String {
        [
            format!("<{}>", self.tag).as_str(),
            target,
            format!("</{}>", self.tag).as_str()
        ].iter().map(|x| *x).collect()
    }
}
let italic = MyMarkup {
    tag: String::from("i"),
};
let bold = MyMarkup {
    tag: String::from("b"),
};
let underline = MyMarkup {
    tag: String::from("u"),
};
let foo: Span<MyMarkup> = Span::new(Cow::Borrowed(&italic), Cow::Owned(String::from("foo")));
let bar: Span<MyMarkup> = Span::new(Cow::Borrowed(&bold), Cow::Owned(String::from("bar")));
let foobar = foo.join(&bar);
assert_eq!(format!("{}", foobar), "<i>foo</i><b>bar</b>");
let foobaz = foobar.replace("bar", "baz");
assert_eq!(format!("{}", foobaz), "<i>foo</i><b>baz</b>");
let mut buz: Spans<MyMarkup> = Default::default();
buz = buz.join(&Span::new(Cow::Borrowed(&underline), Cow::Owned(String::from("buz"))));
let foobuz = foobar.replace("bar", &buz);
assert_eq!(format!("{}", foobuz), "<i>foo</i><u>buz</u>");
let foob = foobar.slice(..4).unwrap();
assert_eq!(format!("{}", foob), "<i>foo</i><b>b</b>");
fn make_spans(style: &MyMarkup, text: &str) -> Spans<MyMarkup> {
    let mut spans: Spans<MyMarkup> = Default::default();
    let span: Span<MyMarkup> = Span::new(Cow::Borrowed(style), Cow::Borrowed(text));
    spans = spans.join(&span);
    spans
}
let truncation = TruncationStyle::Inner(Some(Span::new(Cow::Borrowed(&underline), Cow::Owned(String::from("…")))));
let first_spans = make_spans(&italic, "abcdefg");
let second_spans = make_spans(&bold, "12345678");
let first_segment = TextWidget::new(
    &first_spans,
    &truncation,
);
    
let second_segment = TextWidget::new(
    &second_spans,
    &truncation,
);
let mut hbox: HBox = Default::default();
hbox.push(&first_segment);
hbox.push(&second_segment);
assert_eq!(format!("{}", hbox.truncate(10)), "<i>ab</i><u>…</u><i>fg</i><b>12</b><u>…</u><b>78</b>");
```
