mod graphemes;
mod spans;
mod styled_grapheme;
mod width;
pub use graphemes::*;
pub use spans::*;
use std::fmt;
pub use styled_grapheme::*;
pub use width::*;

use std::ops::{Bound, RangeBounds};
pub trait Text<'a>: fmt::Display + Graphemes<'a> + HasWidth {
    fn raw(&self) -> String;
    fn slice_width(
        &'a self,
        range: (Bound<usize>, Bound<usize>),
    ) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        Box::new(
            self.graphemes()
                .scan(0, move |position, g| {
                    if let Width::Bounded(w) = g.width() {
                        *position += w;
                        let in_range = range.contains(position);
                        Some((g, in_range))
                    } else {
                        unreachable!("Grapheme with unbounded width!")
                    }
                })
                .skip_while(|(_g, in_range)| !in_range)
                .take_while(|(_g, in_range)| *in_range)
                .map(|(g, _in_range)| g),
        )
    }
}

pub trait HasWidth {
    fn width(&self) -> Width;
}

pub trait FiniteText<'a>: Text<'a> + fmt::Debug {
    fn bounded_width(&'a self) -> usize {
        match self.width() {
            Width::Bounded(w) => w,
            Width::Unbounded => {
                unreachable!("Created a finite text object with an unbounded width")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ansi_term::Color;
    #[test]
    fn ansi_string() {
        let string = "Test";
        let style = Color::Red.normal();
        let ansistring = style.paint(string);
        let expected = StyledGrapheme::borrowed(&style, &string[..1]);
        let actual = ansistring.graphemes().next().unwrap();
        assert_eq!(expected, actual);
    }
    #[test]
    fn ansi_strings() {
        let string = "Test";
        let style = Color::Red.normal();
        let ansistrings = vec![style.paint(string)];
        let expected = StyledGrapheme::borrowed(&style, &string[..1]);
        let actual = ansistrings.graphemes().next().unwrap();
        assert_eq!(expected, actual);
    }
}