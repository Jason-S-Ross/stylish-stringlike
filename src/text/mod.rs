//! This provides the primary text object, [`Spans`], which is a sequence
//! of styled spans, as well as traits providing support for string-like
//! methods on structs.

mod expandable;
mod joinable;
mod painter;
mod pushable;
mod replaceable;
mod sliceable;
mod spans;
mod splitable;
mod width;
mod width_sliceable;
pub use expandable::Expandable;
pub use joinable::Joinable;
pub use painter::*;
pub use pushable::Pushable;
pub use replaceable::*;
pub use sliceable::*;
pub use spans::*;
pub use splitable::*;
pub use width::*;
pub use width_sliceable::*;

/// Support for converting a text object into a raw, unstyled string
pub trait RawText {
    /// Return an owned copy of the raw contents of the text object.
    ///
    /// # Example
    /// ```
    /// use stylish_stringlike::text::RawText;
    /// let foo = String::from("foobar");
    /// assert_eq!(foo.raw(), foo);
    /// ````
    fn raw(&self) -> String;
    /// Return a reference ot the raw contents of the text object.
    ///
    /// # Example
    /// ```
    /// use stylish_stringlike::text::RawText;
    /// let foo = String::from("foobar");
    /// assert_eq!(foo.raw_ref(), &foo);
    /// ```
    fn raw_ref(&self) -> &str;
}

impl RawText for String {
    fn raw(&self) -> String {
        self.clone()
    }
    fn raw_ref(&self) -> &str {
        self.as_str()
    }
}

impl RawText for &str {
    fn raw(&self) -> String {
        self.to_string()
    }
    fn raw_ref(&self) -> &str {
        *self
    }
}
