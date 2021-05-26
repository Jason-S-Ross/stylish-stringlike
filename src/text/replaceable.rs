use super::{Expandable, Pushable, RawText, Sliceable};
use regex::Regex;
/// Replacing text in text-like objects
pub trait Replaceable<'a, T> {
    /// Perform literal string replacement.
    fn replace(&'a self, from: &str, replacer: T) -> Self;
    /// Perform regex string replacement.
    fn replace_regex(&'a self, searcher: &Regex, replacer: T) -> Self;
}

impl<'a, T> Replaceable<'a, &'a T> for T
where
    T: Default + RawText + Sliceable<'a> + Pushable<T> + Expandable,
{
    fn replace(&'a self, from: &str, replacer: &'a T) -> Self {
        let mut result: T = Default::default();
        let mut last_end = 0;
        for (start, part) in self.raw_ref().match_indices(from) {
            match self.slice(last_end..start) {
                Some(slice) if !slice.raw_ref().is_empty() => {
                    result.push(&slice);
                    result.push(replacer);
                }
                _ => {}
            }
            last_end = start + part.len();
        }
        match self.slice(last_end..) {
            Some(slice) if !slice.raw_ref().is_empty() => {
                result.push(&slice);
            }
            _ => {}
        }
        result
    }
    fn replace_regex(&'a self, searcher: &Regex, replacer: &'a T) -> Self {
        let mut result: T = Default::default();
        let mut last_end = 0;
        let captures = searcher.captures_iter(self.raw_ref());
        for capture in captures {
            let mat = capture
                .get(0)
                .expect("Captures are always supposed to have at least one match");
            if let Some(slice) = self.slice(last_end..mat.start()) {
                result.push(&slice);
                if let Some(_original) = self.slice(mat.start()..mat.end()) {
                    let expanded = replacer.expand(&capture);
                    result.push(&expanded);
                }
            }
            last_end = mat.end();
        }
        if let Some(spans) = self.slice(last_end..) {
            result.push(&spans);
        }
        result
    }
}
