pub trait Splitable<'a, T> {
    type Delim;
    type Result;
    #[allow(clippy::type_complexity)]
    fn split(
        &'a self,
        pattern: T,
    ) -> Box<dyn Iterator<Item = (Option<Self::Result>, Option<Self::Delim>)> + 'a>;
}
