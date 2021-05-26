use regex::Captures;

/// Expanding regex captures in text objects.
pub trait Expandable {
    /// Returns self with the desired capture group expanded into self.
    /// For [`String`], this is just a wrapper around [`Captures::expand`].
    ///
    /// # Example
    /// ```rust
    /// use regex::Regex;
    /// use stylish_stringlike::text::Expandable;
    /// let re = Regex::new(r"(\d{3})[-. ,](\d{3})[-. ,](\d{4})").unwrap();
    /// let captures = re.captures("555,123 4567").unwrap();
    /// let target = String::from("($1) $2-$3");
    /// assert_eq!(target.expand(&captures), String::from("(555) 123-4567"))
    /// ```
    fn expand(&self, capture: &Captures) -> Self;
}

impl Expandable for String {
    fn expand(&self, capture: &Captures) -> String {
        let mut dest = String::new();
        capture.expand(self, &mut dest);
        dest
    }
}
