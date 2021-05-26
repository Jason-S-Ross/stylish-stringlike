/// Trait for text objects that can have content pushed into them without changing type.
pub trait Pushable<T: ?Sized> {
    /// Pushes another text object onto this one.
    fn push(&mut self, other: &T);
}
