use crate::segmentation::{GraphemeIndex, UnicodeSegmentation, UnicodeWord, UnicodeWordIndex};
use ansi_term::{ANSIString, ANSIStrings, Style};
use std::fmt;
use std::ops::Index;
use std::iter::FromIterator;
use std::slice::SliceIndex;
use unic_segment::{Graphemes, WordBoundIndices};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug)]
pub struct StyledGrapheme<'a> {
    grapheme: &'a str,
    style: Style,
}

impl<'a> StyledGrapheme<'a> {
    pub fn width(&self) -> usize {
        // self.grapheme.width_cjk()
        1
    }
}

impl<'a> From<&'a StyledGrapheme<'a>> for ANSIString<'a> {
    fn from(grapheme: &'a StyledGrapheme<'a>) -> ANSIString<'a> {
        grapheme.style.paint(grapheme.grapheme)
    }
}

#[derive(Debug, Clone)]
pub struct Span<'a> {
    content: Vec<StyledGrapheme<'a>>,
}

impl<'a> From<&'a ANSIString<'a>> for Span<'a> {
    fn from(string: &'a ANSIString<'a>) -> Span<'a> {
        let style = *string.style_ref();
        let content = Graphemes::new(string)
            .map(|grapheme| StyledGrapheme { grapheme, style })
            .collect();

        Span {content}
    }
}


impl<'a> FromIterator<&'a ANSIString<'a>> for Span<'a> {
    fn from_iter<I: IntoIterator<Item=&'a ANSIString<'a>>>(iter: I) -> Span<'a> {
        let mut content: Vec<StyledGrapheme> = vec![];
        for string in iter {
            let style = *string.style_ref();
            for grapheme in Graphemes::new(string) {
                content.push(StyledGrapheme { grapheme, style })
            }
        }
        Span{ content }
    }
}

impl<'a> FromIterator<&'a StyledGrapheme<'a>> for Span<'a> {
    fn from_iter<I: IntoIterator<Item=&'a StyledGrapheme<'a>>>(iter: I) -> Span<'a> {
        let mut content: Vec<StyledGrapheme> = vec![];
        for grapheme in iter {
            content.push(grapheme.clone())
        }
        Span { content }
    }
}

impl<'a> Span<'a> {
    pub fn new(content: Vec<StyledGrapheme<'a>>) -> Span<'a> {
        Span { content }
    }
    fn as_plain_string(&self) -> String {
        self.content.iter().map(|x| x.grapheme.to_owned()).collect()
    }
    pub fn width(&self) -> usize {
        // Because element in content is one grapheme, width is length
        self.content.iter().map(|x| x.width()).sum()
    }
}

impl<'a, T> Index<T> for Span<'a>
where
    T: SliceIndex<[StyledGrapheme<'a>]>,
{
    type Output = T::Output;
    fn index(&self, index: T) -> &Self::Output {
        &self.content[index]
    }
}

impl<'a> fmt::Display for Span<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut strings: Vec<ANSIString> = vec![];
        for grapheme in &self.content {
            strings.push(grapheme.into())
        }
        let strings = ANSIStrings(&strings);
        strings.fmt(f)
    }
}

impl<'a> UnicodeSegmentation<StyledGrapheme<'a>, [StyledGrapheme<'a>]> for Span<'a> {

    const EXTENDED: bool = false;
    fn graphemes(&self) -> Vec<StyledGrapheme<'a>> {
        // Spans are build from unicode graphemes already so this is trivial
        self.content.clone()
    }
    fn grapheme_indices(&self) -> Vec<GraphemeIndex<StyledGrapheme<'a>>> {
        // Spans are build from unicode graphemes already so this is trivial
        self.content
            .iter()
            .enumerate()
            .map(|(index, grapheme)| GraphemeIndex::new(index, grapheme))
            .collect()
    }
    fn unicode_words(&self) -> Vec<UnicodeWord<[StyledGrapheme<'a>]>> {
        // This implementation is hard because the unicode words don't split the way
        // i'd like. I can't just use split_word_indices.
        todo!("Unicode Words")
    }
    fn split_word_bound_indices(&self) -> Vec<UnicodeWordIndex<[StyledGrapheme<'a>]>> {
        eprintln!("split_word_bound: {}", self);
        let mut start = 0;
        let mut result = vec![];
        for (_index, word) in WordBoundIndices::new(&self.as_plain_string()) {
            let grapheme_count = Graphemes::new(word).count();
            let end = start + grapheme_count;
            result.push(UnicodeWordIndex::new(start, &self[start..end]));
            start = end;
        }
        result
    }
}
