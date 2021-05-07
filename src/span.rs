use crate::segmentation::{GraphemeIndex, UnicodeSegmentation, UnicodeWord, UnicodeWordIndex};
use ansi_term::{ANSIString, ANSIStrings, Style};
use std::fmt;
use std::ops::Index;
use std::iter::FromIterator;
use std::slice::SliceIndex;
use unicode_segmentation::UnicodeSegmentation as USeg;

#[derive(Clone, Copy, Debug)]
pub struct StyledGrapheme<'a> {
    grapheme: &'a str,
    style: Style,
}

impl<'a> From<&'a StyledGrapheme<'a>> for ANSIString<'a> {
    fn from(grapheme: &'a StyledGrapheme<'a>) -> ANSIString<'a> {
        grapheme.style.paint(grapheme.grapheme)
    }
}

#[derive(Debug)]
pub struct Span<'a> {
    content: Vec<StyledGrapheme<'a>>,
}

impl<'a> From<&'a ANSIString<'a>> for Span<'a> {
    fn from(string: &'a ANSIString<'a>) -> Span<'a> {
        let style = *string.style_ref();
        Span {
            content: string
                .graphemes(true)
                .map(|grapheme| StyledGrapheme { grapheme, style })
                .collect(),
        }
    }
}


impl<'a> FromIterator<&'a ANSIString<'a>> for Span<'a> {
    fn from_iter<I: IntoIterator<Item=&'a ANSIString<'a>>>(iter: I) -> Span<'a> {
        let mut content: Vec<StyledGrapheme> = vec![];
        for string in iter {
            let style = *string.style_ref();
            for grapheme in string.graphemes(true) {
                content.push(StyledGrapheme { grapheme, style })
            }
        }
        Span{ content }
    }
}

impl<'a> Span<'a> {
    fn as_plain_string(&self) -> String {
        self.content.iter().map(|x| x.grapheme.to_owned()).collect()
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

    const EXTENDED: bool = true;
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
        let mut start = 0;
        let mut result = vec![];
        for (_index, word) in self.as_plain_string().split_word_bound_indices() {
            let grapheme_count = word.graphemes(true).count();
            let end = start + grapheme_count;
            result.push(UnicodeWordIndex::new(start, &self[start..end]));
            start = end;
        }
        result
    }
}
