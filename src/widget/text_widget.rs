use crate::text::{FiniteText, HasWidth, StyledGrapheme, Width};
use crate::widget::{Truncatable, TruncationStyle};

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
    pub fn truncate(&self, width: usize) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        use TruncationStyle::{Inner, Left, Outer, Right};
        match self.truncation_style {
            Left => self.text.truncate_left(width, self.truncation_symbol),
            Right => self.text.truncate_right(width, self.truncation_symbol),
            Inner => self.text.truncate_inner(width, self.truncation_symbol),
            Outer => self.text.truncate_outer(width, self.truncation_symbol),
        }
    }
}

impl<'a> HasWidth for TextWidget<'a> {
    fn width(&self) -> Width {
        self.text.width()
    }
}
