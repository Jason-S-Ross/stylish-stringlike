pub mod hbox;
pub mod repeat;
pub mod text_widget;
pub mod truncatable;
pub use hbox::*;
pub use repeat::*;
pub use text_widget::*;
pub use truncatable::*;

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::Spans;
    use crate::text::StyledGrapheme;
    use ansi_term::{ANSIStrings, Color};
    #[test]
    fn truncate_trivial_left() {
        let style = Color::Red.normal();
        let content = "0123456";
        let string = style.paint(content);
        let text: Spans = (&string).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Left, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(4).collect::<Spans>());
        let expected = format!("{}", ANSIStrings(&[style.paint("012"), ellipsis]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_left() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let texts = vec![style0.paint(content0), style1.paint(content1)];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Left, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(4).collect::<Spans>());
        let expected = format!("{}", ANSIStrings(&[style0.paint("012"), ellipsis]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_2_left() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let texts = vec![style0.paint(content0), style1.paint(content1)];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Left, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(9).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[style0.paint(content0), style1.paint("7"), ellipsis])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_two_widgets_first_left() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "01234";
        let content1 = "56789";
        let first_text = style0.paint(content0);
        let first_span: Spans = (&first_text).into();
        let second_text = style1.paint(content1);
        let second_span: Spans = (&second_text).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget_container = vec![
            TextWidget::new(&first_span, TruncationStyle::Left, &ellipsis_span),
            TextWidget::new(&second_span, TruncationStyle::Left, &ellipsis_span),
        ];
        let widgets = widget_container.iter().collect::<Vec<_>>();
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(8).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                style0.paint("012"),
                ellipsis.clone(),
                style1.paint("567"),
                ellipsis.clone(),
            ])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_left_noop() {
        let style = Color::Red.normal();
        let content = "0123456";
        let string = style.paint(content);
        let text: Spans = (&string).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Left, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(7).collect::<Spans>());
        let expected = format!("{}", ANSIStrings(&[string]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn trunctate_infinite_left() {
        let repeat_widget =
            Repeat::new(StyledGrapheme::owned(Color::Blue.normal(), "=".to_string()));
        let truncator = Color::Black.paint(".");
        let truncator_span: Spans = (&truncator).into();
        let repeat_text_widget =
            TextWidget::new(&repeat_widget, TruncationStyle::Left, &truncator_span);
        let widgets = vec![&repeat_text_widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(5).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[Color::Blue.normal().paint("===="), truncator.clone(),])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_right() {
        let style = Color::Red.normal();
        let content = "0123456";
        let string = style.paint(content);
        let text: Spans = (&string).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Right, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(4).collect::<Spans>());
        let expected = format!("{}", ANSIStrings(&[ellipsis, style.paint("456")]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_right() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let texts = vec![style0.paint(content0), style1.paint(content1)];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Right, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(6).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[ellipsis, style0.paint("56"), style1.paint("789"),])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_2_right() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let texts = vec![style0.paint(content0), style1.paint(content1)];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Right, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(9).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[ellipsis, style0.paint("23456"), style1.paint(content1),])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_two_widgets_first_right() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "01234";
        let content1 = "56789";
        let first_text = style0.paint(content0);
        let first_span: Spans = (&first_text).into();
        let second_text = style1.paint(content1);
        let second_span: Spans = (&second_text).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget_container = vec![
            TextWidget::new(&first_span, TruncationStyle::Right, &ellipsis_span),
            TextWidget::new(&second_span, TruncationStyle::Right, &ellipsis_span),
        ];
        let widgets = widget_container.iter().collect::<Vec<_>>();
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(8).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                ellipsis.clone(),
                style0.paint("234"),
                ellipsis.clone(),
                style1.paint("789"),
            ])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_right_noop() {
        let style = Color::Red.normal();
        let content = "0123456";
        let string = style.paint(content);
        let text: Spans = (&string).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Right, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(7).collect::<Spans>());
        let expected = format!("{}", ANSIStrings(&[string]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn trunctate_infinite_right() {
        let repeat_widget =
            Repeat::new(StyledGrapheme::owned(Color::Blue.normal(), "=".to_string()));
        let truncator = Color::Black.paint(".");
        let truncator_span: Spans = (&truncator).into();
        let repeat_text_widget =
            TextWidget::new(&repeat_widget, TruncationStyle::Right, &truncator_span);
        let widgets = vec![&repeat_text_widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(5).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[truncator.clone(), Color::Blue.normal().paint("===="),])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_inner() {
        let style = Color::Red.normal();
        let content = "0123456";
        let string = style.paint(content);
        let text: Spans = (&string).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Inner, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(4).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[style.paint("01"), ellipsis, style.paint("6")])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_inner() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "01234";
        let content1 = "56789";
        let texts = vec![style0.paint(content0), style1.paint(content1)];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Inner, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(6).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[style0.paint("012"), ellipsis, style1.paint("89"),])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_2_inner() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let texts = vec![style0.paint(content0), style1.paint(content1)];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Inner, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(9).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                style0.paint("0123"),
                ellipsis,
                style0.paint("6"),
                style1.paint(content1),
            ])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_two_widgets_first_inner() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "01234";
        let content1 = "56789";
        let first_text = style0.paint(content0);
        let first_span: Spans = (&first_text).into();
        let second_text = style1.paint(content1);
        let second_span: Spans = (&second_text).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget_container = vec![
            TextWidget::new(&first_span, TruncationStyle::Inner, &ellipsis_span),
            TextWidget::new(&second_span, TruncationStyle::Inner, &ellipsis_span),
        ];
        let widgets = widget_container.iter().collect::<Vec<_>>();
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(8).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                style0.paint("01"),
                ellipsis.clone(),
                style0.paint("4"),
                style1.paint("56"),
                ellipsis.clone(),
                style1.paint("9"),
            ])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_inner_noop() {
        let style = Color::Red.normal();
        let content = "0123456";
        let string = style.paint(content);
        let text: Spans = (&string).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Inner, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(7).collect::<Spans>());
        let expected = format!("{}", ANSIStrings(&[string]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn trunctate_infinite_inner() {
        let repeat_widget =
            Repeat::new(StyledGrapheme::owned(Color::Blue.normal(), "=".to_string()));
        let truncator = Color::Black.paint(".");
        let truncator_span: Spans = (&truncator).into();
        let repeat_text_widget =
            TextWidget::new(&repeat_widget, TruncationStyle::Inner, &truncator_span);
        let widgets = vec![&repeat_text_widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(5).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                Color::Blue.normal().paint("=="),
                truncator.clone(),
                Color::Blue.normal().paint("=="),
            ])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_outer_odd_even() {
        let style = Color::Red.normal();
        let content = "0123456";
        let string = style.paint(content);
        let text: Spans = (&string).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Outer, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(4).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[ellipsis.clone(), style.paint("23"), ellipsis.clone(),])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_outer_even_even() {
        let style = Color::Red.normal();
        let content = "01234567";
        let string = style.paint(content);
        let text: Spans = (&string).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Outer, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(4).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[ellipsis.clone(), style.paint("34"), ellipsis.clone(),])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_outer_even_odd() {
        let style = Color::Red.normal();
        let content = "01234567";
        let string = style.paint(content);
        let text: Spans = (&string).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Outer, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(5).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[ellipsis.clone(), style.paint("234"), ellipsis.clone(),])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_outer_odd_odd() {
        let style = Color::Red.normal();
        let content = "0123456";
        let string = style.paint(content);
        let text: Spans = (&string).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Outer, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(5).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[ellipsis.clone(), style.paint("234"), ellipsis.clone(),])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_outer() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "01234";
        let content1 = "56789";
        let texts = vec![style0.paint(content0), style1.paint(content1)];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Outer, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(6).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                ellipsis.clone(),
                style0.paint("34"),
                style1.paint("56"),
                ellipsis.clone(),
            ])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_2_outer() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let texts = vec![style0.paint(content0), style1.paint(content1)];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Outer, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(8).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                ellipsis.clone(),
                style0.paint("23456"),
                style1.paint("7"),
                ellipsis.clone(),
            ])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_two_widgets_first_outer() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "01234";
        let content1 = "56789";
        let first_text = style0.paint(content0);
        let first_span: Spans = (&first_text).into();
        let second_text = style1.paint(content1);
        let second_span: Spans = (&second_text).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget_container = vec![
            TextWidget::new(&first_span, TruncationStyle::Outer, &ellipsis_span),
            TextWidget::new(&second_span, TruncationStyle::Outer, &ellipsis_span),
        ];
        let widgets = widget_container.iter().collect::<Vec<_>>();
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(8).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                ellipsis.clone(),
                style0.paint("12"),
                ellipsis.clone(),
                ellipsis.clone(),
                style1.paint("67"),
                ellipsis.clone(),
            ])
        );
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_trivial_outer_noop() {
        let style = Color::Red.normal();
        let content = "0123456";
        let string = style.paint(content);
        let text: Spans = (&string).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Outer, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(7).collect::<Spans>());
        let expected = format!("{}", ANSIStrings(&[string]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn trunctate_infinite_outer() {
        let repeat_widget =
            Repeat::new(StyledGrapheme::owned(Color::Blue.normal(), "=".to_string()));
        let truncator = Color::Black.paint(".");
        let truncator_span: Spans = (&truncator).into();
        let repeat_text_widget =
            TextWidget::new(&repeat_widget, TruncationStyle::Outer, &truncator_span);
        let widgets = vec![&repeat_text_widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(5).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                truncator.clone(),
                Color::Blue.normal().paint("==="),
                truncator.clone(),
            ])
        );
        assert_eq!(expected, actual);
    }
}
