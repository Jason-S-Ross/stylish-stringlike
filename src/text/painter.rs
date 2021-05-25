use ansi_term::{ANSIStrings, Style};
use std::borrow::Borrow;
/// Provides functionality to display strings with markup.
pub trait Painter {
    /// Applies markup to a given string.
    ///
    /// # Example
    ///
    /// ```
    /// use stylish_stringlike::text::Painter;
    /// struct MyMarkup {
    ///     opening_tag: String,
    ///     closing_tag: String,
    /// }
    /// impl Painter for MyMarkup {
    ///     fn paint(&self, target: &str) -> String {
    ///         [&self.opening_tag, target, &self.closing_tag].iter().map(|x| *x).collect()
    ///     }
    /// }
    /// let italic = MyMarkup {
    ///     opening_tag: String::from("<i>"),
    ///     closing_tag: String::from("</i>"),
    /// };
    /// assert_eq!(italic.paint("foo"), String::from("<i>foo</i>"));
    /// ```
    fn paint(&self, target: &str) -> String;
    /// Applies markup to a given iterator of ([`Painter`], [`str`]) objects.
    /// Provide an implementation for this if multiple adjacent [`Painter`]s
    /// can be joined together.
    ///
    /// # Example
    /// ```
    /// use stylish_stringlike::text::Painter;
    /// use std::borrow::Borrow;
    /// struct MyMarkup {
    ///     opening_tag: String,
    ///     closing_tag: String,
    /// }
    /// impl Painter for MyMarkup {
    ///     fn paint(&self, target: &str) -> String {
    ///         [&self.opening_tag, target, &self.closing_tag].iter().map(|x| *x).collect()
    ///     }
    ///     fn paint_many<'a, T, U, V>(groups: T) -> String
    ///     where
    ///         T: IntoIterator<Item = (U, V)> + 'a,
    ///         U: Borrow<Self> + 'a,
    ///         V: Borrow<str> + 'a,
    ///     {
    ///         let mut result = String::new();
    ///         let mut previous_open = None;
    ///         let mut previous_close: Option<String> = None;
    ///         for (paintable, s) in groups {
    ///             if Some(paintable.borrow().opening_tag.to_string()) != previous_open {
    ///                 if let Some(s) = previous_close {
    ///                     result.push_str(s.as_str());
    ///                 }
    ///                 result.push_str(&paintable.borrow().opening_tag);
    ///                 previous_open = Some(paintable.borrow().opening_tag.to_string());
    ///                 previous_close = Some(paintable.borrow().closing_tag.to_string());
    ///             }
    ///             result.push_str(s.borrow());
    ///         }
    ///         if let Some(s) = previous_close {
    ///             result.push_str(s.as_str())
    ///         }
    ///         result
    ///     }
    ///     
    /// }
    /// let italic = MyMarkup {
    ///     opening_tag: String::from("<i>"),
    ///     closing_tag: String::from("</i>"),
    /// };
    /// let bold = MyMarkup {
    ///     opening_tag: String::from("<b>"),
    ///     closing_tag: String::from("</b>"),
    /// };
    /// let foobarbaz = vec![
    ///     (&italic, "foo"),
    ///     (&italic, "bar"),
    ///     (&bold, "baz"),
    /// ];
    /// assert_eq!(MyMarkup::paint_many(foobarbaz), String::from("<i>foobar</i><b>baz</b>"));
    /// ```
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
