use crate::text::{HasWidth, Width};
use crate::widget::{Truncateable, TruncationStrategy};

/// Widgets that can be truncated to fit in a provided width.
pub trait Fitable<'a, T: Truncateable<'a>>: HasWidth {
    /// Truncate self to fit in a given width.
    fn truncate(&self, width: usize) -> Option<T>;
}

/// A widget that can be truncated
pub struct TextWidget<'a, T, U> {
    text: &'a T,
    truncation_strategy: &'a U,
}

impl<'a, T, U> TextWidget<'a, T, U> {
    pub fn new(text: &'a T, truncation_strategy: &'a U) -> Self {
        TextWidget {
            text,
            truncation_strategy,
        }
    }
}

impl<'a, T, U> Fitable<'a, T::Output> for TextWidget<'a, T, U>
where
    T: Truncateable<'a>,
    U: TruncationStrategy<'a, T>,
    T::Output: Truncateable<'a> + HasWidth,
{
    fn truncate(&self, width: usize) -> Option<T::Output> {
        self.truncation_strategy.truncate(self.text, width)
    }
}

impl<'a, T, U> HasWidth for TextWidget<'a, T, U>
where
    T: HasWidth,
{
    fn width(&self) -> Width {
        self.text.width()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::*;
    use crate::widget::TruncationStyle;
    use std::borrow::Cow;
    #[test]
    fn truncate_widget() {
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
        let widget = TextWidget::new(&spans, &truncator);
        let actual = format!("{}", widget.truncate(9).unwrap());
        let expected = String::from("<2>01234</2><3>5</3><1>...</1>");
        assert_eq!(expected, actual);
    }
}
