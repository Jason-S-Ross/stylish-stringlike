use ansi_term::{ANSIStrings, Style};
use std::borrow::Borrow;
pub(crate) trait Painter {
    fn paint(&self, target: &str) -> String;
    fn paint_many<'a, T, U, V>(groups: T) -> String
    where
        T: IntoIterator<Item = (U, V)> + 'a,
        U: Borrow<Self> + 'a,
        V: Borrow<str> + 'a,
    {
        let mut result = String::new();
        for (painter, text) in groups {
            result.push_str(&painter.borrow().paint(text.borrow()));
        }
        result
    }
}
impl Painter for Style {
    fn paint(&self, target: &str) -> String {
        Style::paint(*self, target).to_string()
    }
    fn paint_many<'a, T, U, V>(groups: T) -> String
    where
        T: IntoIterator<Item = (U, V)> + 'a,
        U: Borrow<Style> + 'a,
        V: Borrow<str> + 'a,
    {
        let mut strings = vec![];
        for (style, text) in groups {
            let text = text.borrow().to_string();
            strings.push(Style::paint(*style.borrow(), text));
        }
        format!("{}", ANSIStrings(strings.as_slice()))
    }
}
