use regex::Regex;
/// Replacing text in text-like objects
pub trait Replaceable<'a, T> {
    /// Perform literal string replacement.
    fn replace(&'a self, from: &str, replacer: T) -> Self;
    /// Perform regex string replacement.
    fn replace_regex(&'a self, searcher: &Regex, replacer: T) -> Self;
}
