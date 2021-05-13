use crate::text::{FiniteText, HasWidth, StyledGrapheme, Text, Width};
use std::fmt;
use std::ops::Bound;

#[allow(dead_code)]
pub trait Truncatable<'a>: fmt::Display + HasWidth + fmt::Debug {
    fn truncate_left(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a>;
    fn truncate_right(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a>;
    fn truncate_outer(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a>;
    fn truncate_inner(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a>;
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum TruncationStyle {
    #[allow(dead_code)]
    Left,
    #[allow(dead_code)]
    Right,
    #[allow(dead_code)]
    Inner,
    #[allow(dead_code)]
    Outer,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TextWidget<'a> {
    text: &'a dyn Truncatable<'a>,
    truncation_style: TruncationStyle,
    truncation_symbol: &'a dyn FiniteText<'a>,
}

impl<'a> TextWidget<'a> {
    #[allow(dead_code)]
    pub fn new(
        text: &'a dyn Truncatable<'a>,
        truncation_style: TruncationStyle,
        truncation_symbol: &'a dyn FiniteText<'a>,
    ) -> Self {
        TextWidget {
            text,
            truncation_style,
            truncation_symbol,
        }
    }
    #[allow(dead_code)]
    fn truncate(&self, width: usize) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        use TruncationStyle::{Inner, Left, Outer, Right};
        match self.truncation_style {
            Left => self.text.truncate_left(width, self.truncation_symbol),
            Right => self.text.truncate_right(width, self.truncation_symbol),
            Inner => self.text.truncate_inner(width, self.truncation_symbol),
            Outer => self.text.truncate_outer(width, self.truncation_symbol),
        }
    }
}

impl<'a, T> Truncatable<'a> for T
where
    T: Text<'a> + fmt::Debug,
{
    fn truncate_left(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        let self_width = self.width();
        match self_width {
            Width::Bounded(w) if width >= w => {
                return self.graphemes();
            }
            _ => {}
        }
        Box::new(
            self.slice_width((
                Bound::Unbounded,
                Bound::Included(width.saturating_sub(symbol.bounded_width())),
            ))
            .chain(symbol.graphemes()),
        )
    }
    fn truncate_right(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        let self_width = self.width();
        match self_width {
            Width::Bounded(w) if width >= w => {
                return self.graphemes();
            }
            _ => {}
        }
        if let Width::Bounded(self_width) = self_width {
            Box::new(symbol.graphemes().chain(self.slice_width((
                Bound::Excluded(
                    self_width.saturating_sub(width.saturating_sub(symbol.bounded_width())),
                ),
                Bound::Unbounded,
            ))))
        } else {
            Box::new(symbol.graphemes().chain(self.slice_width((
                Bound::Unbounded,
                Bound::Included(width.saturating_sub(symbol.bounded_width())),
            ))))
        }
    }
    fn truncate_outer(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        let self_width = self.width();
        match self_width {
            Width::Bounded(w) if width >= w => {
                return self.graphemes();
            }
            _ => {}
        }
        let sym_width = symbol.bounded_width();
        if let Width::Bounded(self_width) = self_width {
            let diff = self_width.saturating_sub(width) + 2 * sym_width;
            let start = diff / 2;
            let end = start + width.saturating_sub(2 * sym_width);
            Box::new(
                symbol
                    .graphemes()
                    .chain(self.slice_width((Bound::Excluded(start), Bound::Included(end))))
                    .chain(symbol.graphemes()),
            )
        } else {
            Box::new(
                symbol
                    .graphemes()
                    .chain(
                        self.slice_width((
                            Bound::Unbounded,
                            Bound::Included(width - 2 * sym_width),
                        )),
                    )
                    .chain(symbol.graphemes()),
            )
        }
    }
    fn truncate_inner(
        &'a self,
        width: usize,
        symbol: &'a dyn FiniteText<'a>,
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        let self_width = self.width();
        match self_width {
            Width::Bounded(w) if width >= w => {
                return self.graphemes();
            }
            _ => {}
        }
        let sym_width = symbol.bounded_width();
        let text_width = width.saturating_sub(sym_width);
        let w = text_width / 2;
        if let Width::Bounded(self_width) = self_width {
            Box::new(
                self.slice_width((Bound::Unbounded, Bound::Included(w + text_width % 2)))
                    .chain(symbol.graphemes())
                    .chain(self.slice_width((
                        Bound::Excluded(self_width.saturating_sub(w)),
                        Bound::Unbounded,
                    ))),
            )
        } else {
            Box::new(
                self.slice_width((Bound::Unbounded, Bound::Included(w + text_width % 2)))
                    .chain(symbol.graphemes())
                    .chain(self.slice_width((Bound::Unbounded, Bound::Included(w)))),
            )
        }
    }
}

#[allow(dead_code)]
pub struct HBox<'a> {
    elements: Vec<&'a TextWidget<'a>>,
}

impl<'a> HBox<'a> {
    #[allow(dead_code)]
    pub fn new(elements: &[&'a TextWidget<'a>]) -> Self {
        HBox {
            elements: elements.to_vec(),
        }
    }
    #[allow(dead_code)]
    pub fn truncate(&'a self, width: usize) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        let mut space = width;
        let mut todo: Vec<(usize, _)> = self
            .elements
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                if let Width::Bounded(_w) = element.text.width() {
                    Some((index, element))
                } else {
                    None
                }
            })
            .collect();
        let mut to_fit = todo.len();
        let mut widths: std::collections::HashMap<usize, usize> = Default::default();
        while to_fit > 0 {
            let target_width: f32 = space as f32 / to_fit as f32;
            let mut to_pop = vec![];
            for (rel_index, (index, element)) in todo.iter().enumerate() {
                if let Width::Bounded(w) = element.text.width() {
                    if (w as f32) <= target_width {
                        space -= w;
                        to_fit -= 1;
                        widths.insert(*index, w);
                        to_pop.push(rel_index)
                    }
                }
            }
            for index in to_pop.iter().rev() {
                todo.remove(*index);
            }
            if to_pop.is_empty() {
                let target_width = space / todo.len();
                let rem = space % todo.len();
                for (i, (index, _widget)) in todo.iter().enumerate() {
                    let w = if i < rem {
                        target_width + 1
                    } else {
                        target_width
                    };
                    space -= w;
                    widths.insert(*index, w);
                }
                break;
            }
        }
        let infinite_widths: Vec<(usize, _)> = self
            .elements
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                if let Width::Unbounded = element.text.width() {
                    Some((index, element))
                } else {
                    None
                }
            })
            .collect();
        if !infinite_widths.is_empty() {
            let target_width = space / infinite_widths.len();
            let rem = space % infinite_widths.len();
            for (rel_index, (abs_index, _element)) in infinite_widths.iter().enumerate() {
                let w = if rel_index < rem {
                    target_width + 1
                } else {
                    target_width
                };
                widths.insert(*abs_index, w);
            }
        }

        Box::new(
            self.elements
                .iter()
                .enumerate()
                .flat_map(move |(i, widget)| widget.truncate(widths[&i])),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ansi_term::{ANSIStrings, Color};
    use crate::text::Spans;
    use crate::repeat::Repeat;
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
        let texts = vec![
            style0.paint(content0),
            style1.paint(content1),
        ];
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
        let texts = vec![
            style0.paint(content0),
            style1.paint(content1),
        ];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Left, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(9).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                style0.paint(content0),
                style1.paint("7"),
                ellipsis
            ]));
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
        let repeat_widget = Repeat::new(StyledGrapheme::owned(Color::Blue.normal(), "=".to_string()));
        let truncator = Color::Black.paint(".");
        let truncator_span: Spans = (&truncator).into();
        let repeat_text_widget = TextWidget::new(&repeat_widget, TruncationStyle::Left, &truncator_span);
        let widgets = vec![&repeat_text_widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(5).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                Color::Blue.normal().paint("===="),
                truncator.clone(),
            ])
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
        let texts = vec![
            style0.paint(content0),
            style1.paint(content1),
        ];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Right, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(6).collect::<Spans>());
        let expected = format!("{}", ANSIStrings(&[
            ellipsis,
            style0.paint("56"),
            style1.paint("789"),
        ]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_2_right() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let texts = vec![
            style0.paint(content0),
            style1.paint(content1),
        ];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Right, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(9).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                ellipsis,
                style0.paint("23456"),
                style1.paint(content1),
            ]));
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
        let repeat_widget = Repeat::new(StyledGrapheme::owned(Color::Blue.normal(), "=".to_string()));
        let truncator = Color::Black.paint(".");
        let truncator_span: Spans = (&truncator).into();
        let repeat_text_widget = TextWidget::new(&repeat_widget, TruncationStyle::Right, &truncator_span);
        let widgets = vec![&repeat_text_widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(5).collect::<Spans>());
        let expected = format!(
            "{}",
            ANSIStrings(&[
                truncator.clone(),
                Color::Blue.normal().paint("===="),
            ])
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
        let expected = format!("{}", ANSIStrings(&[
            style.paint("01"),
            ellipsis,
            style.paint("6")
        ]));
        assert_eq!(expected, actual);

    }
    #[test]
    fn truncate_compound_span_inner() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "01234";
        let content1 = "56789";
        let texts = vec![
            style0.paint(content0),
            style1.paint(content1),
        ];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Inner, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(6).collect::<Spans>());
        let expected = format!("{}", ANSIStrings(&[
            style0.paint("012"),
            ellipsis,
            style1.paint("89"),
        ]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_2_inner() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let texts = vec![
            style0.paint(content0),
            style1.paint(content1),
        ];
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
            ]));
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
        let repeat_widget = Repeat::new(StyledGrapheme::owned(Color::Blue.normal(), "=".to_string()));
        let truncator = Color::Black.paint(".");
        let truncator_span: Spans = (&truncator).into();
        let repeat_text_widget = TextWidget::new(&repeat_widget, TruncationStyle::Inner, &truncator_span);
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
        let expected = format!("{}", ANSIStrings(&[
            ellipsis.clone(),
            style.paint("23"),
            ellipsis.clone(),
        ]));
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
        let expected = format!("{}", ANSIStrings(&[
            ellipsis.clone(),
            style.paint("34"),
            ellipsis.clone(),
        ]));
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
        let expected = format!("{}", ANSIStrings(&[
            ellipsis.clone(),
            style.paint("234"),
            ellipsis.clone(),
        ]));
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
        let expected = format!("{}", ANSIStrings(&[
            ellipsis.clone(),
            style.paint("234"),
            ellipsis.clone(),
        ]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_outer() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "01234";
        let content1 = "56789";
        let texts = vec![
            style0.paint(content0),
            style1.paint(content1),
        ];
        let text: Spans = (&texts).into();
        let ellipsis = Color::Blue.paint("…");
        let ellipsis_span: Spans = (&ellipsis).into();
        let widget = TextWidget::new(&text, TruncationStyle::Outer, &ellipsis_span);
        let widgets = vec![&widget];
        let hbox = HBox::new(&widgets);
        let actual = format!("{}", hbox.truncate(6).collect::<Spans>());
        let expected = format!("{}", ANSIStrings(&[
            ellipsis.clone(),
            style0.paint("34"),
            style1.paint("56"),
            ellipsis.clone(),
        ]));
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_compound_span_2_outer() {
        let style0 = Color::Red.normal();
        let style1 = Color::Green.normal();
        let content0 = "0123456";
        let content1 = "789";
        let texts = vec![
            style0.paint(content0),
            style1.paint(content1),
        ];
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
            ]));
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
        let repeat_widget = Repeat::new(StyledGrapheme::owned(Color::Blue.normal(), "=".to_string()));
        let truncator = Color::Black.paint(".");
        let truncator_span: Spans = (&truncator).into();
        let repeat_text_widget = TextWidget::new(&repeat_widget, TruncationStyle::Outer, &truncator_span);
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
