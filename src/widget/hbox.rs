use crate::text::{Pushable, Width};
use crate::widget::{Fitable, Truncateable};

/// A displayable box of text widgets.
#[derive(Default)]
pub struct HBox<'a, T: Truncateable> {
    elements: Vec<&'a dyn Fitable<T>>,
}

impl<'a, T: Truncateable> HBox<'a, T> {
    pub fn new() -> Self {
        HBox {
            elements: Vec::new(),
        }
    }
    /// Adds an element.
    pub fn push(&mut self, element: &'a dyn Fitable<T>) {
        self.elements.push(element);
    }
    /// Truncates this widget to a given size.
    pub fn truncate(&'a self, width: usize) -> T
    where
        T: Pushable<T> + Pushable<T::Output> + Default,
    {
        let mut space = width;
        let mut todo: Vec<(usize, _)> = self
            .elements
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                if let Width::Bounded(_w) = element.width() {
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
                if let Width::Bounded(w) = element.width() {
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
                if let Width::Unbounded = element.width() {
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

        let mut res: T = Default::default();
        let elements = self
            .elements
            .iter()
            .enumerate()
            .map(move |(i, widget)| widget.truncate(widths[&i]))
            .flatten();
        for elem in elements {
            res.push(&elem)
        }
        res
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::*;
    use crate::widget::{Repeat, TextWidget, TruncationStyle};
    use std::borrow::Cow;
    #[test]
    fn make_hbox() {
        let fmt_1 = Tag::new("<1>", "</1>");
        let fmt_2 = Tag::new("<2>", "</2>");
        let fmt_3 = Tag::new("<3>", "</3>");
        let mut spans: Spans<Tag> = Default::default();
        spans.push(&Span::new(Cow::Borrowed(&fmt_2), Cow::Borrowed("01234")));
        spans.push(&Span::new(Cow::Borrowed(&fmt_3), Cow::Borrowed("56789")));
        let truncator = {
            let mut ellipsis = Spans::<Tag>::default();
            ellipsis.push(&Span::new(Cow::Borrowed(&fmt_1), Cow::Borrowed("...")));
            TruncationStyle::Left(ellipsis)
        };
        let widget = TextWidget::new(Cow::Borrowed(&spans), Cow::Borrowed(&truncator));
        let mut hbox: HBox<Spans<Tag>> = Default::default();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(9));
        let expected = String::from("<2>01234</2><3>5</3><1>...</1>");
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_hbox_infinite() {
        let fmt_1 = Tag::new("<1>", "</1>");
        let fmt_2 = Tag::new("<2>", "</2>");
        let span = Span::new(Cow::Borrowed(&fmt_2), Cow::Borrowed("="));
        let repeat = Repeat::new(span);
        let truncator =
            TruncationStyle::Left(Span::new(Cow::Borrowed(&fmt_1), Cow::Borrowed("...")));
        let widget = TextWidget::new(Cow::Borrowed(&repeat), Cow::Borrowed(&truncator));
        let mut hbox: HBox<Spans<Tag>> = Default::default();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(5));
        let expected = String::from("<2>==</2><1>...</1>");
        assert_eq!(expected, actual);
    }
    #[test]
    fn make_hbox_literal() {
        let fmt_2 = Tag::new("<2>", "</2>");
        let fmt_3 = Tag::new("<3>", "</3>");
        let mut spans: Spans<Tag> = Default::default();
        spans.push(&Span::new(Cow::Borrowed(&fmt_2), Cow::Borrowed("01234")));
        spans.push(&Span::new(Cow::Borrowed(&fmt_3), Cow::Borrowed("56789")));
        let truncator = TruncationStyle::Left("...");
        let widget = TextWidget::new(Cow::Borrowed(&spans), Cow::Borrowed(&truncator));
        let mut hbox: HBox<Spans<Tag>> = Default::default();
        hbox.push(&widget);
        let actual = format!("{}", hbox.truncate(9));
        let expected = String::from("<2>01234</2><3>5...</3>");
        assert_eq!(expected, actual);
    }
}
