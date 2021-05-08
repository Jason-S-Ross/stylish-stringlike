#[derive(Debug)]
pub struct GraphemeIndex<'a, G> {
    index: usize,
    grapheme: &'a G,
}

impl<'a, G> GraphemeIndex<'a, G> {
    pub fn new(index: usize, grapheme: &'a G) -> GraphemeIndex<'a, G> {
        GraphemeIndex { index, grapheme }
    }
}

#[derive(Debug)]
pub struct UnicodeWord<'a, W: ?Sized> {
    word: &'a W,
}

impl<'a, W: ?Sized> UnicodeWord<'a, W> {
    pub fn new(word: &'a W) -> UnicodeWord<'a, W> {
        UnicodeWord { word }
    }
}

#[derive(Debug)]
pub struct UnicodeWordIndex<'a, W: ?Sized> {
    index: usize,
    word: UnicodeWord<'a, W>,
}
impl<'a, W: ?Sized> UnicodeWordIndex<'a, W> {
    pub fn new(index: usize, word: &'a W) -> UnicodeWordIndex<'a, W> {
        let word = UnicodeWord::new(word);
        UnicodeWordIndex { index, word }
    }
}

pub trait UnicodeSegmentation<G, W: ?Sized> {
    const EXTENDED: bool;
    fn graphemes(&self) -> Vec<G>;
    fn grapheme_indices(&self) -> Vec<GraphemeIndex<G>>;
    fn unicode_words(&self) -> Vec<UnicodeWord<W>>;
    fn split_word_bound_indices(&self) -> Vec<UnicodeWordIndex<W>>;
}
