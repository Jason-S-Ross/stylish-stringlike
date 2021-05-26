mod hbox;
mod repeat;
mod text_widget;
mod truncatable;
pub use hbox::*;
pub use repeat::*;
pub use text_widget::*;
pub use truncatable::*;

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::{Pushable, Span, Spans, WidthSliceable};
    use crate::widget::truncatable::TruncationStyle;
    use ansi_term::{ANSIStrings, Color, Style};
    use std::borrow::Cow;
    fn make_spans(style: &Style, text: &str) -> Spans<Style> {
        let ansistring = style.paint(text);
        let span: Span<Style> = ansistring.into();
        let mut spans: Spans<Style> = Default::default();
        spans.push(&span);
        spans
    }
    #[test]
    fn truncate_trivial_left() {
        let style = Color::Red.normal();
        let content = "0123456";
        let text = make_spans(&style, content);
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Left(Some(ellipsis_span));
        let widget = TextWidget::new(&text, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(4));
        let expected = format!("{}{}", style.paint("012"), ellipsis_style.paint(ellipsis));
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_left() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let text = {
            let mut text = make_spans(&style0, content0);
            text.push(&make_spans(&style1, content1));
            text
        };
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Left(Some(ellipsis_span));
        let widget = TextWidget::new(&text, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(4));
        let expected = format!("{}{}", style0.paint("012"), ellipsis_style.paint(ellipsis));
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_2_left() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let text = {
            let mut text = make_spans(&style0, content0);
            text.push(&make_spans(&style1, content1));
            text
        };
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Left(Some(ellipsis_span));
        let widget = TextWidget::new(&text, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(9));
        let expected = format!(
            "{}{}",
            ANSIStrings(&[style0.paint(content0), style1.paint("7"),]),
            ellipsis_style.paint(ellipsis)
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_two_widgets_first_left() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "01234";
        let content1 = "56789";
        let first_span = make_spans(&style0, content0);
        let second_span = make_spans(&style1, content1);
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Left(Some(ellipsis_span));
        let widgets = vec![
            TextWidget::new(&first_span, &truncation),
            TextWidget::new(&second_span, &truncation),
        ];
        let mut hbox = HBox::new();
        hbox.push(&widgets[0]);
        hbox.push(&widgets[1]);
        let actual = format!("{}", hbox.truncate(8));
        let expected = format!(
            "{}{}{}{}",
            style0.paint("012"),
            ellipsis_style.paint(ellipsis),
            style1.paint("567"),
            ellipsis_style.paint(ellipsis),
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_left_noop() {
        let style = Color::Red.normal();
        let content = "0123456";
        let string = style.paint(content);
        let text = make_spans(&style, content);
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Left(Some(ellipsis_span));
        let widget = TextWidget::new(&text, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(7));
        let expected = format!("{}", ANSIStrings(&[string]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn trunctate_infinite_left() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Blue.normal()),
            Cow::Owned("=".to_string()),
        );
        use std::ops::Bound;
        span.slice_width((Bound::Unbounded, Bound::Included(5)));
        let repeat_widget = Repeat::new(span);
        let truncator_style = Color::Black.normal();
        let truncator_text = ".";
        let truncator_span = make_spans(&truncator_style, truncator_text);
        let truncation = TruncationStyle::Left(Some(truncator_span));
        let repeat_text_widget = TextWidget::new(&repeat_widget, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&repeat_text_widget);
        let actual = format!("{}", hbox.truncate(5));
        let expected = format!(
            "{}{}",
            Color::Blue.normal().paint("===="),
            truncator_style.paint(truncator_text),
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_right() {
        let style = Color::Red.normal();
        let content = "0123456";
        let text = make_spans(&style, content);
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Right(Some(ellipsis_span));
        let widget = TextWidget::new(&text, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(4));
        let expected = format!("{}{}", ellipsis_style.paint(ellipsis), style.paint("456"));
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_right() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let text = {
            let mut text = make_spans(&style0, content0);
            text.push(&make_spans(&style1, content1));
            text
        };
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Right(Some(ellipsis_span));
        let widget = TextWidget::new(&text, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(6));
        let expected = format!(
            "{}{}",
            ellipsis_style.paint(ellipsis),
            ANSIStrings(&[style0.paint("56"), style1.paint("789"),])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_2_right() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let text = {
            let mut text = make_spans(&style0, content0);
            text.push(&make_spans(&style1, content1));
            text
        };
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Right(Some(ellipsis_span));
        let widget = TextWidget::new(&text, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(9));
        let expected = format!(
            "{}{}",
            ellipsis_style.paint(ellipsis),
            ANSIStrings(&[style0.paint("23456"), style1.paint(content1),])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_two_widgets_first_right() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "01234";
        let content1 = "56789";
        let first_span = make_spans(&style0, content0);
        let second_span = make_spans(&style1, content1);
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Right(Some(ellipsis_span));
        let widget_container = vec![
            TextWidget::new(&first_span, &truncation),
            TextWidget::new(&second_span, &truncation),
        ];
        let mut hbox = HBox::new();
        hbox.push(&widget_container[0]);
        hbox.push(&widget_container[1]);
        let actual = format!("{}", hbox.truncate(8));
        let expected = format!(
            "{}{}{}{}",
            ellipsis_style.paint(ellipsis),
            style0.paint("234"),
            ellipsis_style.paint(ellipsis),
            style1.paint("789"),
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_right_noop() {
        let style = Color::Red.normal();
        let content = "0123456";
        let string = style.paint(content);
        let text = make_spans(&style, content);
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Right(Some(ellipsis_span));
        let widget = TextWidget::new(&text, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(7));
        let expected = format!("{}", ANSIStrings(&[string]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn trunctate_infinite_right() {
        let repeat_widget = Repeat::new(Span::<Style>::new(
            Cow::Owned(Color::Blue.normal()),
            Cow::Owned("=".to_string()),
        ));
        let truncator_style = Color::Black.normal();
        let truncator_str = ".";
        let truncator_span = make_spans(&truncator_style, truncator_str);
        let truncation = TruncationStyle::Right(Some(truncator_span));
        let repeat_text_widget = TextWidget::new(&repeat_widget, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&repeat_text_widget);
        let actual = format!("{}", hbox.truncate(5));
        let expected = format!(
            "{}{}",
            truncator_style.paint(truncator_str),
            Color::Blue.normal().paint("===="),
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_inner() {
        let style = Color::Red.normal();
        let content = "0123456";
        let text = make_spans(&style, content);
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Inner(Some(ellipsis_span));
        let widget = TextWidget::new(&text, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(4));
        let expected = format!(
            "{}{}{}",
            style.paint("01"),
            ellipsis_style.paint(ellipsis),
            style.paint("6")
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_inner() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "01234";
        let content1 = "56789";
        let text = {
            let mut text = make_spans(&style0, content0);
            text.push(&make_spans(&style1, content1));
            text
        };
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Inner(Some(ellipsis_span));
        let widget = TextWidget::new(&text, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(6));
        let expected = format!(
            "{}{}{}",
            style0.paint("012"),
            ellipsis_style.paint(ellipsis),
            style1.paint("89"),
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_2_inner() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let text = {
            let mut text = make_spans(&style0, content0);
            text.push(&make_spans(&style1, content1));
            text
        };
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Inner(Some(ellipsis_span));
        let widget = TextWidget::new(&text, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(9));
        let expected = format!(
            "{}{}{}",
            style0.paint("0123"),
            ellipsis_style.paint(ellipsis),
            ANSIStrings(&[style0.paint("6"), style1.paint(content1),])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_two_widgets_first_inner() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "01234";
        let content1 = "56789";
        let first_span = make_spans(&style0, content0);
        let second_span = make_spans(&style1, content1);
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Inner(Some(ellipsis_span));
        let widgets = vec![
            TextWidget::new(&first_span, &truncation),
            TextWidget::new(&second_span, &truncation),
        ];
        let mut hbox = HBox::new();
        hbox.push(&widgets[0]);
        hbox.push(&widgets[1]);
        let actual = format!("{}", hbox.truncate(8));
        let expected = format!(
            "{}{}{}{}{}{}",
            style0.paint("01"),
            ellipsis_style.paint(ellipsis),
            style0.paint("4"),
            style1.paint("56"),
            ellipsis_style.paint(ellipsis),
            style1.paint("9"),
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_inner_noop() {
        let style = Color::Red.normal();
        let content = "0123456";
        let text = make_spans(&style, content);
        let ellipsis_style = Color::Blue.normal();
        let ellipsis = "…";
        let ellipsis_span = make_spans(&ellipsis_style, ellipsis);
        let truncation = TruncationStyle::Inner(Some(ellipsis_span));
        let widget = TextWidget::new(&text, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(7));
        let expected = format!("{}", ANSIStrings(&[style.paint(content)]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn trunctate_infinite_inner() {
        let repeat_widget = Repeat::new(Span::<Style>::new(
            Cow::Owned(Color::Blue.normal()),
            Cow::Owned("=".to_string()),
        ));
        let truncator_style = Color::Black.normal();
        let truncator_text = ".";
        let truncator = truncator_style.paint(truncator_text);
        let truncator_span = make_spans(&truncator_style, truncator_text);
        let truncation = TruncationStyle::Inner(Some(truncator_span));
        let repeat_text_widget = TextWidget::new(&repeat_widget, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&repeat_text_widget);
        let actual = format!("{}", hbox.truncate(5));
        let expected = format!(
            "{}{}{}",
            Color::Blue.normal().paint("=="),
            truncator.clone(),
            Color::Blue.normal().paint("=="),
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn trunctate_infinite_none_left() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Blue.normal()),
            Cow::Owned("=".to_string()),
        );
        use std::ops::Bound;
        span.slice_width((Bound::Unbounded, Bound::Included(5)));
        let repeat_widget = Repeat::new(span);
        let truncator_style = Color::Black.normal();
        let truncator_text = ".";
        let truncator_span = make_spans(&truncator_style, truncator_text);
        let truncation = TruncationStyle::Left(Some(truncator_span));
        let repeat_text_widget = TextWidget::new(&repeat_widget, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&repeat_text_widget);
        let actual = format!("{}", hbox.truncate(0));
        let expected = String::new();
        assert_eq!(expected, actual);
    }
    #[test]
    fn trunctate_infinite_none_right() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Blue.normal()),
            Cow::Owned("=".to_string()),
        );
        use std::ops::Bound;
        span.slice_width((Bound::Unbounded, Bound::Included(5)));
        let repeat_widget = Repeat::new(span);
        let truncator_style = Color::Black.normal();
        let truncator_text = ".";
        let truncator_span = make_spans(&truncator_style, truncator_text);
        let truncation = TruncationStyle::Right(Some(truncator_span));
        let repeat_text_widget = TextWidget::new(&repeat_widget, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&repeat_text_widget);
        let actual = format!("{}", hbox.truncate(0));
        let expected = String::new();
        assert_eq!(expected, actual);
    }
    #[test]
    fn trunctate_infinite_none_inner() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Blue.normal()),
            Cow::Owned("=".to_string()),
        );
        use std::ops::Bound;
        span.slice_width((Bound::Unbounded, Bound::Included(5)));
        let repeat_widget = Repeat::new(span);
        let truncator_style = Color::Black.normal();
        let truncator_text = ".";
        let truncator_span = make_spans(&truncator_style, truncator_text);
        let truncation = TruncationStyle::Inner(Some(truncator_span));
        let repeat_text_widget = TextWidget::new(&repeat_widget, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&repeat_text_widget);
        let actual = format!("{}", hbox.truncate(0));
        let expected = String::new();
        assert_eq!(expected, actual);
    }
    #[test]
    fn trunctate_infinite_only_symbol() {
        let span = Span::<Style>::new(
            Cow::Owned(Color::Blue.normal()),
            Cow::Owned("=".to_string()),
        );
        use std::ops::Bound;
        span.slice_width((Bound::Unbounded, Bound::Included(5)));
        let repeat_widget = Repeat::new(span);
        let truncator_style = Color::Black.normal();
        let truncator_text = ".";
        let truncator_span = make_spans(&truncator_style, truncator_text);
        let truncation = TruncationStyle::Inner(Some(truncator_span));
        let repeat_text_widget = TextWidget::new(&repeat_widget, &truncation);
        let mut hbox = HBox::new();
        hbox.push(&repeat_text_widget);
        let actual = format!("{}", hbox.truncate(1));
        let expected = format!("{}", truncator_style.paint(truncator_text));
        assert_eq!(expected, actual);
    }
}
