use regex::Regex;
pub trait Replaceable<'a, T> {
    fn replace(&'a self, from: &str, replacer: T) -> Self;
    fn replace_regex(&'a self, searcher: &Regex, replacer: T) -> Self;
}
