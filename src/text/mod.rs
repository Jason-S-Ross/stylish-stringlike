//! This provides the primary text object, [`spans`], which is a sequence
//! of styled spans, as well as traits providing support for string-like
//! methods on structs.

mod joinable;
mod painter;
mod replaceable;
mod sliceable;
mod spans;
mod splitable;
mod width;
mod width_sliceable;
pub use joinable::Joinable;
pub use painter::*;
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
