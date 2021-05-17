use crate::text::{FiniteText, HasWidth, StyledGrapheme, Width};
use crate::widget::{Truncatable, TruncationStyle};

#[allow(dead_code)]
pub struct TextWidget<'a, T: Clone> {
    text: &'a dyn Truncatable<'a, T>,
    truncation_style: TruncationStyle,
    truncation_symbol: &'a dyn FiniteText<'a, T>,
}

impl<'a, T: Clone> TextWidget<'a, T> {
    #[allow(dead_code)]
    pub fn new(
        text: &'a dyn Truncatable<'a, T>,
        truncation_style: TruncationStyle,
        truncation_symbol: &'a dyn FiniteText<'a, T>,
    ) -> Self {
        TextWidget {
            text,
            truncation_style,
            truncation_symbol,
        }
    }
    #[allow(dead_code)]
    pub fn truncate(&self, width: usize) -> Box<dyn Iterator<Item = StyledGrapheme<'a, T>> + 'a> {
        use TruncationStyle::{Inner, Left, Outer, Right};
        match self.truncation_style {
            Left => self.text.truncate_left(width, self.truncation_symbol),
            Right => self.text.truncate_right(width, self.truncation_symbol),
            Inner => self.text.truncate_inner(width, self.truncation_symbol),
            Outer => self.text.truncate_outer(width, self.truncation_symbol),
        }
    }
}

impl<'a, T: Clone> HasWidth for TextWidget<'a, T> {
    fn width(&self) -> Width {
        self.text.width()
    }
}
