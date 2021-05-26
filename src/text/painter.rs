#[cfg(test)]
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
    ///     tag: String,
    /// }
    ///
    /// impl Painter for MyMarkup {
    ///     fn paint(&self, target: &str) -> String {
    ///         [
    ///             format!("<{}>", self.tag).as_str(),
    ///             target,
    ///             format!("</{}>", self.tag).as_str(),
    ///         ]
    ///         .iter()
    ///         .map(|x| *x)
    ///         .collect()
    ///     }
    /// }
    /// let italic = MyMarkup {
    ///     tag: String::from("i"),
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
    /// use std::borrow::Borrow;
    /// use stylish_stringlike::text::Painter;
    /// #[derive(Clone, Eq, PartialEq)]
    /// struct MyMarkup {
    ///     tag: String,
    /// }
    ///
    /// impl Painter for MyMarkup {
    ///     fn paint(&self, target: &str) -> String {
    ///         [
    ///             format!("<{}>", self.tag).as_str(),
    ///             target,
    ///             format!("</{}>", self.tag).as_str(),
    ///         ]
    ///         .iter()
    ///         .map(|x| *x)
    ///         .collect()
    ///     }
    ///     fn paint_many<'a, T, U, V>(groups: T) -> String
    ///     where
    ///         T: IntoIterator<Item = (U, V)> + 'a,
    ///         U: Borrow<Self> + 'a,
    ///         V: Borrow<str> + 'a,
    ///     {
    ///         let mut result = String::new();
    ///         let mut previous_span = String::new();
    ///         let mut previous_tag: Option<MyMarkup> = None;
    ///         for (painter, s) in groups {
    ///             match previous_tag {
    ///                 Some(ref p) if painter.borrow() != p => {
    ///                     result += &p.paint(&previous_span);
    ///                     previous_span = String::from(s.borrow());
    ///                     previous_tag = Some(painter.borrow().clone());
    ///                 }
    ///                 Some(ref p) => {
    ///                     previous_span.push_str(s.borrow());
    ///                 },
    ///                 None => {
    ///                     previous_span.push_str(s.borrow());
    ///                     previous_tag = Some(painter.borrow().clone());
    ///                 }
    ///             }
    ///         }
    ///         if let Some(p) = previous_tag {
    ///             if !previous_span.is_empty() {
    ///                 result += &p.paint(&previous_span);
    ///             }
    ///         }
    ///         result
    ///     }
    /// }
    /// let italic = MyMarkup {
    ///     tag: String::from("i"),
    /// };
    /// let bold = MyMarkup {
    ///     tag: String::from("b"),
    /// };
    /// let foobarbaz = vec![(&italic, "foo"), (&italic, "bar"), (&bold, "baz")];
    /// assert_eq!(
    ///     MyMarkup::paint_many(foobarbaz),
    ///     String::from("<i>foobar</i><b>baz</b>")
    /// );
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

#[cfg(test)]
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
