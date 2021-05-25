use crate::text::{HasWidth, Width};
use crate::widget::{Truncateable, TruncationStrategy};
use std::fmt::Display;

/// Widgets that can be truncated to fit in a provided width.
pub trait Fitable: HasWidth {
    /// Truncate self to fit in a given width.
    fn truncate(&self, width: usize) -> Option<String>;
}

/// A widget that can be truncated
pub struct TextWidget<'a, T, U>
where
    T: Truncateable<'a>,
    T::Output: Display,
    U: TruncationStrategy<'a, T>,
{
    text: &'a T,
    truncation_strategy: &'a U,
}

impl<'a, T, U> TextWidget<'a, T, U>
where
    T: Truncateable<'a>,
    T::Output: Display,
    U: TruncationStrategy<'a, T>,
{
    pub fn new(text: &'a T, truncation_strategy: &'a U) -> Self {
        TextWidget {
            text,
            truncation_strategy,
        }
    }
}

impl<'a, T, U> Fitable for TextWidget<'a, T, U>
where
    T: Truncateable<'a>,
    T::Output: Display,
    U: TruncationStrategy<'a, T>,
{
    fn truncate(&self, width: usize) -> Option<String> {
        self.truncation_strategy.truncate(self.text, width)
    }
}

impl<'a, T, U> HasWidth for TextWidget<'a, T, U>
where
    T: Truncateable<'a>,
    T::Output: Display,
    U: TruncationStrategy<'a, T>,
{
    fn width(&self) -> Width {
        self.text.width()
    }
}
