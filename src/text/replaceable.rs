use regex::{Regex, Replacer};
pub trait Replaceable<T> {
    type Output;
    fn replace(&self, from: T, to: T) -> Result<Self::Output, ()>;
    fn replace_regex<R: Replacer>(&self, searcher: &Regex, replacer: R)
        -> Result<Self::Output, ()>;
}
