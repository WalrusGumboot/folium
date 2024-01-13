use crate::{
    ast::{
        AbstractElement, AbstractElementData, AbstractElementID, ElementType, GlobalState, Slide,
    },
    style::{extract_number, extract_size_spec, StyleMap, StyleTarget},
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
            h: self.h - 2 * margin,
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
pub struct SizeSpec {
    pub width: Option<u32>,
    pub height: Option<u32>,
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
        // TODO: take names into account!!!!!
        match self.data() {
            AbstractElementData::Sized(elem) => {
                let size_spec = extract_size_spec(
                    style_map
                        .styles_for_target(&StyleTarget::Named(self.name().clone().unwrap()))
                        .unwrap(),
                    "size",
                );

                let used_width = if let Some(width) = size_spec.width {
                    if area.w < width {
                        eprintln!("warning: specified width was bigger than available");
                        area.w
                    } else {
                        width
                    }
                } else {
                    area.w
                };

                let used_height = if let Some(height) = size_spec.height {
                    if area.h < height {
                        eprintln!("warning: specified height was bigger than available");
                        area.h
                    } else {
                        height
                    }
                } else {
                    area.h
                };

                Vec::from(&[LayoutElement {
                    element: *elem,
                    max_bounds: Rect {
                        x: area.x,
                        y: area.y,
                        w: used_width,
                        h: used_height,
                    },
                }])
            }
            AbstractElementData::Row(elems) => {
                let row_gap = extract_number(
                    style_map
                        .styles_for_target(&StyleTarget::Anonymous(ElementType::Row))
                        .expect("no style map for rows was found"),
                    "gap",
                );

                let sized_elements = elems
                    .iter()
                    .flat_map(|id| global.get_element_by_id(*id))
                    .filter(|elem| elem.el_type() == ElementType::Sized)
                    .collect::<Vec<_>>();

                let all_widths = sized_elements
                    .iter()
                    .flat_map(|elem| {
                        extract_size_spec(
                            style_map
                                .styles_for_target(&StyleTarget::Named(
                                    elem.name().clone().unwrap(),
                                ))
                                .unwrap(),
                            "size",
                        )
                        .width
                    })
                    .collect::<Vec<_>>();

                let total_sized_width = all_widths.iter().sum::<u32>();

                if total_sized_width + row_gap * (elems.len() - 1) as u32 > area.w {
                    panic!("The specified layout will always overflow.")
                }

                let remaining_space = area.w - total_sized_width;

                let single_el_width = (remaining_space - (elems.len() - 1) as u32 * row_gap)
                    / (elems.len() - sized_elements.len()) as u32;

                let mut x_coord = area.x;
                elems
                    .iter()
                    .flat_map(|el| global.get_element_by_id(*el))
                    .flat_map(|elem| {
                        let bounds = if sized_elements.contains(&elem) {
                            let spec = extract_size_spec(
                                style_map
                                    .styles_for_target(&StyleTarget::Named(
                                        elem.name().clone().unwrap(),
                                    ))
                                    .unwrap(),
                                "size",
                            );

                            if let Some(width) = spec.width {
                                Rect {
                                    x: x_coord,
                                    y: area.y,
                                    w: width,
                                    h: spec.height.unwrap_or(area.h),
                                }
                            } else {
                                Rect {
                                    x: x_coord,
                                    y: area.y,
                                    w: single_el_width,
                                    h: spec.height.unwrap_or(area.h),
                                }
                            }
                        } else {
                            Rect {
                                x: x_coord,
                                y: area.y,
                                w: single_el_width,
                                h: area.h,
                            }
                        };

                        x_coord += bounds.w + row_gap;

                        elem.layout(global, style_map, bounds)
                    })
                    .collect()
            }
            AbstractElementData::Col(elems) => {
                let col_gap = extract_number(
                    style_map
                        .styles_for_target(&StyleTarget::Anonymous(ElementType::Col))
                        .expect("no style map for columns was found"),
                    "gap",
                );

                let sized_elements = elems
                    .iter()
                    .flat_map(|id| global.get_element_by_id(*id))
                    .filter(|elem| elem.el_type() == ElementType::Sized)
                    .collect::<Vec<_>>();

                let all_heights = sized_elements
                    .iter()
                    .flat_map(|elem| {
                        extract_size_spec(
                            style_map
                                .styles_for_target(&StyleTarget::Named(
                                    elem.name().clone().unwrap(),
                                ))
                                .unwrap(),
                            "size",
                        )
                        .height
                    })
                    .collect::<Vec<_>>();

                let total_sized_height = all_heights.iter().sum::<u32>();

                if total_sized_height + col_gap * (elems.len() - 1) as u32 > area.h {
                    panic!("The specified layout will always overflow.")
                }

                let remaining_space = area.h - total_sized_height;

                let single_el_height = (remaining_space - (elems.len() - 1) as u32 * col_gap)
                    / (elems.len() - sized_elements.len()) as u32;

                let mut y_coord = area.y;
                elems
                    .iter()
                    .flat_map(|el| global.get_element_by_id(*el))
                    .flat_map(|elem| {
                        let bounds = if sized_elements.contains(&elem) {
                            let spec = extract_size_spec(
                                style_map
                                    .styles_for_target(&StyleTarget::Named(
                                        elem.name().clone().unwrap(),
                                    ))
                                    .unwrap(),
                                "size",
                            );

                            if let Some(height) = spec.height {
                                Rect {
                                    x: area.x,
                                    y: y_coord,
                                    w: spec.width.unwrap_or(area.w),
                                    h: height,
                                }
                            } else {
                                Rect {
                                    x: area.x,
                                    y: y_coord,
                                    w: spec.width.unwrap_or(area.w),
                                    h: single_el_height,
                                }
                            }
                        } else {
                            Rect {
                                x: area.x,
                                y: y_coord,
                                w: area.w,
                                h: single_el_height,
                            }
                        };

                        y_coord += bounds.h + col_gap;

                        elem.layout(global, style_map, bounds)
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
