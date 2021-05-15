use super::*;
use ansi_term::Style;
use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Debug)]
pub struct StyledGrapheme<'a> {
    style: Cow<'a, Style>,
    grapheme: Cow<'a, str>,
}

#[cfg(test)]
impl<'a> PartialEq for StyledGrapheme<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.style == other.style && self.grapheme == other.grapheme
    }
}

impl<'a> StyledGrapheme<'a> {
    pub fn borrowed(style: &'a Style, grapheme: &'a str) -> Self {
        StyledGrapheme {
            style: Cow::Borrowed(style),
            grapheme: Cow::Borrowed(grapheme),
        }
    }
    #[allow(dead_code)]
    pub fn owned(style: Style, grapheme: String) -> Self {
        StyledGrapheme {
            style: Cow::Owned(style),
            grapheme: Cow::Owned(grapheme),
        }
    }
    pub fn raw(&self) -> String {
        self.grapheme.to_string()
    }
    pub fn grapheme(&self) -> &Cow<'a, str> {
        &self.grapheme
    }
    pub fn style(&self) -> &Cow<'a, Style> {
        &self.style
    }
}

impl<'a> HasWidth for StyledGrapheme<'a> {
    fn width(&self) -> Width {
        Width::Bounded(self.grapheme.width())
    }
}

impl<'a> fmt::Display for StyledGrapheme<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.style.paint(self.grapheme.as_ref()).fmt(fmt)
    }
}

impl<'a> RawText for StyledGrapheme<'a> {
    fn raw(&self) -> String {
        self.grapheme.to_string()
    }
    fn raw_ref<'b>(&self) -> &str {
        &self.grapheme
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ansi_term::Color;
    #[test]
    fn test_grapheme_raw() {
        let foo = Color::Blue.paint("foo");
        let g = foo.graphemes().next().unwrap();
        assert_eq!(g.raw(), "f");
    }
    #[test]
    fn test_grapheme_width() {
        let foo = Color::Blue.paint("a⛄👩");
        let mut graphemes = foo.graphemes();
        assert_eq!(graphemes.next().unwrap().width(), Width::Bounded(1));
        assert_eq!(graphemes.next().unwrap().width(), Width::Bounded(2));
        assert_eq!(graphemes.next().unwrap().width(), Width::Bounded(2));
    }
    #[test]
    fn test_grapheme_fmt() {
        let foo = Color::Blue.paint("foo");
        let g = foo.graphemes().next().unwrap();
        assert_eq!(format!("{}", g), format!("{}", Color::Blue.paint("f")));
    }
}
