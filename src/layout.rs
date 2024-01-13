use crate::{
    ast::{
        AbstractElement, AbstractElementData, AbstractElementID, ElementType, GlobalState, Slide,
    },
    style::{extract_number, StyleMap, StyleTarget},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Default)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn with_margin(&self, margin: u32) -> Self {
        Self {
            x: self.x + margin,
            y: self.y + margin,
            w: self.w - 2 * margin,
            h: self.h - 2 * margin
        }
    }
}

pub fn folium_to_sdl_rect(folium_rect: Rect) -> sdl2::rect::Rect {
    sdl2::rect::Rect::new(
        folium_rect.x as i32,
        folium_rect.y as i32,
        folium_rect.w,
        folium_rect.h,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutElement {
    pub element: AbstractElementID,
    pub max_bounds: Rect,
}

impl AbstractElement {
    pub fn layout(
        &self,
        global: &GlobalState,
        style_map: &StyleMap,
        area: Rect,
    ) -> Vec<LayoutElement> {
        match self.data() {
            AbstractElementData::Row(elems) => {
                let row_gap = extract_number(
                    style_map
                        .styles_for_target(&StyleTarget::Anonymous(ElementType::Row))
                        .expect("no style map for rows was found"),
                    "gap",
                );
                let single_el_width =
                    (area.w - (elems.len() - 1) as u32 * row_gap) / elems.len() as u32;

                elems
                    .iter()
                    .enumerate()
                    .flat_map(|(el_idx, el)| {
                        let bounds = Rect {
                            x: area.x + (single_el_width + row_gap) * el_idx as u32,
                            y: area.y,
                            w: single_el_width,
                            h: area.h,
                        };

                        // row needs recursive call
                        global
                            .get_element_by_id(*el)
                            .unwrap()
                            .layout(global, style_map, bounds)
                    })
                    .collect()
            }
            AbstractElementData::Col(elems) => {
                let col_gap = extract_number(
                    style_map
                        .styles_for_target(&StyleTarget::Anonymous(ElementType::Col))
                        .expect("no style map for rows was found"),
                    "gap",
                );
                let single_el_height =
                    (area.h - ((elems.len() as u32 - 1) - 1) * col_gap) / elems.len() as u32;
                elems
                    .iter()
                    .enumerate()
                    .flat_map(|(el_idx, el)| {
                        let bounds = Rect {
                            x: area.x,
                            y: area.y + (single_el_height + col_gap) * el_idx as u32,
                            w: area.w,
                            h: single_el_height,
                        };

                        global
                            .get_element_by_id(*el)
                            .unwrap()
                            .layout(global, style_map, bounds)
                    })
                    .collect()
            }
            AbstractElementData::Padding(elem) => {
                let padding_amount = extract_number(
                    style_map
                        .styles_for_target(&StyleTarget::Anonymous(ElementType::Padding))
                        .expect("no style map for paddings was found"),
                    "amount",
                );
                let new_bound = area.with_margin(padding_amount);

                global
                    .get_element_by_id(*elem)
                    .unwrap()
                    .layout(global, style_map, new_bound)
            }
            AbstractElementData::Centre(_)
            | AbstractElementData::Text(_)
            | AbstractElementData::Code(_)
            | AbstractElementData::Image(_)
            | AbstractElementData::None => Vec::from(&[LayoutElement {
                max_bounds: area,
                element: self.id(),
            }]),
        }
    }
}

impl Slide {
    /// Layouting a slide positions elements on the slide.
    pub fn layout(&self, global: &GlobalState, size_override: Option<Rect>) -> Vec<LayoutElement> {
        let slide_styles = self
            .style_map()
            .styles_for_target(&StyleTarget::Slide)
            .expect("No default slide style was found.");

        let slide_content = global.get_element_by_id(self.content()).unwrap();

        let base_width = extract_number(slide_styles, "width");
        let base_height = extract_number(slide_styles, "height");
        let slide_margin = extract_number(slide_styles, "margin");

        let area = size_override.unwrap_or(Rect {
            x: slide_margin,
            y: slide_margin,
            w: base_width - 2 * slide_margin,
            h: base_height - 2 * slide_margin,
        });

        slide_content.layout(global, self.style_map(), area)
    }
}
