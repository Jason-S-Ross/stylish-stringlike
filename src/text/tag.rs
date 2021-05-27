use crate::text::Paintable;
use std::borrow::Borrow;

/// A simple format for surrounding text in tags
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tag {
    opening: String,
    closing: String,
}

impl Tag {
    pub fn new(opening: &str, closing: &str) -> Self {
        Self {
            opening: opening.to_string(),
            closing: closing.to_string(),
        }
    }
}

impl Paintable for Tag {
    fn paint(&self, target: &str) -> String {
        [self.opening.as_str(), target, self.closing.as_str()]
            .iter()
            .copied()
            .collect()
    }
    fn paint_many<'a, T, U, V>(groups: T) -> String
    where
        T: IntoIterator<Item = (U, V)> + 'a,
        U: Borrow<Self> + 'a,
        V: Borrow<str> + 'a,
    {
        let mut result = String::new();
        let mut previous_span = String::new();
        let mut previous_tag: Option<Self> = None;
        for (painter, s) in groups {
            match previous_tag {
                Some(ref p) if painter.borrow() != p => {
                    result += &p.paint(&previous_span);
                    previous_span = String::from(s.borrow());
                    previous_tag = Some(painter.borrow().clone());
                }
                Some(ref _p) => {
                    previous_span.push_str(s.borrow());
                }
                None => {
                    previous_span.push_str(s.borrow());
                    previous_tag = Some(painter.borrow().clone());
                }
            }
        }
        if let Some(p) = previous_tag {
            if !previous_span.is_empty() {
                result += &p.paint(&previous_span);
            }
        }
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn tag_text() {
        let fmt_1 = Tag::new("<1>", "</1>");
        let fmt_2 = Tag::new("<2>", "</2>");
        let texts = vec![(&fmt_1, "foo"), (&fmt_2, "bar"), (&fmt_2, "baz")];
        assert_eq!(
            Tag::paint_many(texts),
            String::from("<1>foo</1><2>barbaz</2>")
        );
    }
    #[test]
    fn tag_empty() {
        let texts: Vec<(&Tag, &str)> = vec![];
        assert_eq!(Tag::paint_many(texts), String::new());
    }
}
