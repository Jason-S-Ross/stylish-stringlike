use regex::Captures;

/// Expanding regex captures in text objects.
pub trait Expandable {
    /// Returns self with the desired capture group expanded into self.
    fn expand(&self, capture: &Captures) -> Self;
}

impl Expandable for String {
    fn expand(&self, capture: &Captures) -> String {
        let mut dest = String::new();
        capture.expand(self, &mut dest);
        dest
    }
}
