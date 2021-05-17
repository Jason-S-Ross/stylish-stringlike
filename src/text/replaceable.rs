use regex::{Regex, Replacer};
use std::error::Error;

pub trait Replaceable<T> {
    type Output;
    fn replace(&self, from: T, to: T) -> Result<Self::Output, Box<dyn Error>>;
    fn replace_regex<R: Replacer>(
        &self,
        searcher: &Regex,
        replacer: R,
    ) -> Result<Self::Output, Box<dyn Error>>;
}
