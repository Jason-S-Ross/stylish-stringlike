use crate::text::{BoundedWidth, HasWidth, Pushable, Width, WidthSliceable};

/// Objects that have width and are sliceable on width are truncateable.
pub trait Truncateable: HasWidth + WidthSliceable {}

impl<'a, T> Truncateable for T where T: WidthSliceable + HasWidth {}

/// Functionality for truncating objects using some strategy.
pub trait TruncationStrategy<'a, T>
where
    T: WidthSliceable + HasWidth,
{
    /// Truncates target to width. Output should have a width equal to width.
    fn truncate(&'a self, target: &'a T, width: usize) -> Option<T::Output>;
}

/// Styles for simple truncation.
pub enum TruncationStyle<T: BoundedWidth> {
    /// Keeps the left text, truncates text on the right. Optional symbol added when truncation occurs.
    #[allow(dead_code)]
    Left(T),
    /// Keeps the right text, truncates text on the left. Optional symbol added when truncation occurs.
    #[allow(dead_code)]
    Right(T),
    /// Keeps the outside text, truncates text on the inside. Optional symbol added when truncation occurs.
    #[allow(dead_code)]
    Inner(T),
}

impl<'a, T, S> TruncationStrategy<'a, T> for TruncationStyle<S>
where
    T: Truncateable,
    S: BoundedWidth + WidthSliceable,
    T::Output: Pushable<T::Output> + Pushable<S::Output> + Default + WidthSliceable,
{
    fn truncate(&'a self, target: &'a T, width: usize) -> Option<T::Output> {
        if width == 0 {
            return None;
        }
        use TruncationStyle::*;
        let mut result: T::Output = Default::default();
        if let Width::Bounded(w) = target.width() {
            if width >= w {
                result.push(&target.slice_width(..));
                return Some(result);
            }
            match self {
                Left(ref sym) => {
                    result.push(&target.slice_width(..width.saturating_sub(sym.bounded_width())));
                    result.push(&sym.slice_width(..));
                }
                Right(ref sym) => {
                    result.push(&sym.slice_width(..));
                    result.push(&target.slice_width(
                        w.saturating_sub(width.saturating_sub(sym.bounded_width()))..,
                    ));
                }
                Inner(ref sym) => {
                    let inner_width = sym.bounded_width();
                    let target_width = width.saturating_sub(inner_width);
                    let left_width = target_width / 2 + target_width % 2;
                    let right_width = target_width / 2;
                    let left_slice = target.slice_width(..left_width);
                    let right_slice = target.slice_width(w.saturating_sub(right_width)..);
                    result.push(&left_slice);
                    result.push(&sym.slice_width(..));
                    result.push(&right_slice);
                }
            }
        } else {
            match self {
                Left(ref symbol) => {
                    result
                        .push(&target.slice_width(..width.saturating_sub(symbol.bounded_width())));
                    result.push(&symbol.slice_width(..));
                }
                Right(ref symbol) => {
                    result.push(&symbol.slice_width(..));
                    result
                        .push(&target.slice_width(..width.saturating_sub(symbol.bounded_width())));
                }
                Inner(s) => {
                    let inner_width = s.bounded_width();
                    let target_width = width.saturating_sub(inner_width);
                    let left_width = target_width / 2 + target_width % 2;
                    let right_width = target_width / 2;
                    let left_slice = target.slice_width(..left_width);
                    let right_slice = target.slice_width(..right_width);
                    result.push(&left_slice);
                    result.push(&s.slice_width(..));
                    result.push(&right_slice);
                }
            }
            return Some(result);
        }

        Some(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::*;
    use std::borrow::Cow;
    #[test]
    fn truncate_text() {
        let fmt_1 = Tag::new("<1>", "</1>");
        let fmt_2 = Tag::new("<2>", "</2>");
        let fmt_3 = Tag::new("<3>", "</3>");
        let mut spans: Spans<Tag> = Default::default();
        spans.push(&Span::new(Cow::Borrowed(&fmt_2), Cow::Borrowed("01234")));
        spans.push(&Span::new(Cow::Borrowed(&fmt_3), Cow::Borrowed("56789")));
        let truncator = {
            let mut ellipsis = Spans::<Tag>::default();
            ellipsis.push(&Span::new(Cow::Borrowed(&fmt_1), Cow::Borrowed("...")));
            TruncationStyle::Left(ellipsis)
        };
        let actual = format!("{}", truncator.truncate(&spans, 9).unwrap());
        let expected = String::from("<2>01234</2><3>5</3><1>...</1>");
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_none() {
        let fmt_2 = Tag::new("<2>", "</2>");
        let fmt_3 = Tag::new("<3>", "</3>");
        let mut spans: Spans<Tag> = Default::default();
        spans.push(&Span::new(Cow::Borrowed(&fmt_2), Cow::Borrowed("01234")));
        spans.push(&Span::new(Cow::Borrowed(&fmt_3), Cow::Borrowed("56789")));
        let truncator: TruncationStyle<Option<Spans<Tag>>> = TruncationStyle::Left(None);
        let actual = format!("{}", truncator.truncate(&spans, 9).unwrap());
        let expected = String::from("<2>01234</2><3>5678</3>");
        assert_eq!(expected, actual);
    }
    #[test]
    fn truncate_one() {
        let fmt_1 = Tag::new("<1>", "</1>");
        let fmt_2 = Tag::new("<2>", "</2>");
        let mut spans: Spans<Tag> = Default::default();
        spans.push(&Span::new(Cow::Borrowed(&fmt_2), Cow::Borrowed("0")));
        let truncator = {
            let mut ellipsis = Spans::<Tag>::default();
            ellipsis.push(&Span::new(Cow::Borrowed(&fmt_1), Cow::Borrowed("…")));
            TruncationStyle::Left(ellipsis)
        };
        let actual = format!("{}", truncator.truncate(&spans, 1).unwrap());
        let expected = String::from("<2>0</2>");
        assert_eq!(expected, actual);
    }
}
