/// Provides functionality for joining text objects together.
pub trait Joinable<T> {
    type Output: Sized;
    /// Join an object to another object, returning an owned copy.
    ///
    /// # Example
    /// ```
    /// use stylish_stringlike::text::Joinable;
    /// let foo = String::from("foo");
    /// let bar = String::from("bar");
    /// assert_eq!(foo.join(&bar), String::from("foobar"));
    /// ```
    fn join(&self, other: &T) -> Self::Output;
}

impl Joinable<String> for String {
    type Output = String;
    fn join(&self, other: &String) -> Self::Output {
        [self, other].iter().map(|x| x.as_str()).collect::<String>()
    }
}
