use crate::text::Width;
use crate::widget::Fitable;

#[allow(dead_code)]
pub(crate) struct HBox<'a> {
    elements: Vec<&'a dyn Fitable>,
}

impl<'a> HBox<'a> {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        HBox {
            elements: Vec::new(),
        }
    }
    pub(crate) fn push(&mut self, element: &'a dyn Fitable) {
        self.elements.push(element);
    }
    #[allow(dead_code)]
    pub(crate) fn truncate(&'a self, width: usize) -> String {
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

        self.elements
            .iter()
            .enumerate()
            .map(move |(i, widget)| widget.truncate(widths[&i]))
            .flatten()
            .collect()
    }
}
