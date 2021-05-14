use crate::text::{HasWidth, StyledGrapheme, Width};
use crate::widget::TextWidget;

#[allow(dead_code)]
pub struct HBox<'a> {
    elements: Vec<&'a TextWidget<'a>>,
}

impl<'a> HBox<'a> {
    #[allow(dead_code)]
    pub fn new(elements: &[&'a TextWidget<'a>]) -> Self {
        HBox {
            elements: elements.to_vec(),
        }
    }
    #[allow(dead_code)]
    pub fn truncate(&'a self, width: usize) -> Box<dyn Iterator<Item = StyledGrapheme<'a>> + 'a> {
        let mut space = width;
        let mut todo: Vec<(usize, _)> = self
            .elements
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                if let Width::Bounded(_w) = element.width() {
                    Some((index, element))
                } else {
                    None
                }
            })
            .collect();
        let mut to_fit = todo.len();
        let mut widths: std::collections::HashMap<usize, usize> = Default::default();
        while to_fit > 0 {
            let target_width: f32 = space as f32 / to_fit as f32;
            let mut to_pop = vec![];
            for (rel_index, (index, element)) in todo.iter().enumerate() {
                if let Width::Bounded(w) = element.width() {
                    if (w as f32) <= target_width {
                        space -= w;
                        to_fit -= 1;
                        widths.insert(*index, w);
                        to_pop.push(rel_index)
                    }
                }
            }
            for index in to_pop.iter().rev() {
                todo.remove(*index);
            }
            if to_pop.is_empty() {
                let target_width = space / todo.len();
                let rem = space % todo.len();
                for (i, (index, _widget)) in todo.iter().enumerate() {
                    let w = if i < rem {
                        target_width + 1
                    } else {
                        target_width
                    };
                    space -= w;
                    widths.insert(*index, w);
                }
                break;
            }
        }
        let infinite_widths: Vec<(usize, _)> = self
            .elements
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                if let Width::Unbounded = element.width() {
                    Some((index, element))
                } else {
                    None
                }
            })
            .collect();
        if !infinite_widths.is_empty() {
            let target_width = space / infinite_widths.len();
            let rem = space % infinite_widths.len();
            for (rel_index, (abs_index, _element)) in infinite_widths.iter().enumerate() {
                let w = if rel_index < rem {
                    target_width + 1
                } else {
                    target_width
                };
                widths.insert(*abs_index, w);
            }
        }

        Box::new(
            self.elements
                .iter()
                .enumerate()
                .flat_map(move |(i, widget)| widget.truncate(widths[&i])),
        )
    }
}
