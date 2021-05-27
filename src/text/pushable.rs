/// Trait for text objects that can have content pushed into them without changing type.
pub trait Pushable<T: ?Sized> {
    /// Pushes another text object onto this one. [`String`] implements this
    /// trivially.
    ///
    /// # Example
    /// ```rust
    /// use stylish_stringlike::text::Pushable;
    /// let mut foobar = String::from("foo");
    /// let bar = "bar";
    /// Pushable::<str>::push(&mut foobar, &bar);
    /// assert_eq!(foobar, String::from("foobar"));
    /// ```
    fn push(&mut self, other: &T);
}

impl Pushable<str> for String {
    fn push(&mut self, other: &str) {
        self.push_str(other);
    }
}

impl Pushable<String> for String {
    fn push(&mut self, other: &String) {
        self.push_str(other.as_str());
    }
}

impl<S, O: Sized> Pushable<Option<O>> for S
where
    S: Pushable<O>,
{
    fn push(&mut self, other: &Option<O>) {
        match other {
            Some(ref o) => self.push(o),
            None => {}
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn push_string() {
        let mut foobar = String::from("foo");
        let bar = String::from("bar");
        Pushable::<String>::push(&mut foobar, &bar);
        assert_eq!(foobar, String::from("foobar"));
    }
    #[test]
    fn push_string_option() {
        let mut foobar = String::from("foo");
        let bar = Some(String::from("bar"));
        let baz: Option<String> = None;
        Pushable::push(&mut foobar, &bar);
        Pushable::push(&mut foobar, &baz);
        assert_eq!(foobar, String::from("foobar"));
    }
}
