use std::collections::HashMap;

use sdl2::{
    image::LoadTexture,
    render::{Canvas, RenderTarget, Texture, TextureCreator},
    surface::SurfaceContext,
};

use crate::{
    ast::{AbstractElementData, AbstractElementID, ElementType, GlobalState},
    layout::{folium_to_sdl_rect, LayoutElement, Rect},
    style::{extract_colour, extract_number, StyleTarget},
    SLIDE_HEIGHT, SLIDE_WIDTH,
};

pub struct SlideData {
    layout_rects: Vec<LayoutElement>,
    background: (u8, u8, u8),
    pub dimensions: (u32, u32),
}

pub fn generate_slide_data(global: &GlobalState, idx: usize, fullscreen: bool) -> SlideData {
    let slides = global.slides.borrow();
    let slide_styles = slides[idx]
        .style_map()
        .styles_for_target(StyleTarget::Slide)
        .unwrap();

    let background = extract_colour(slide_styles, "bg");
    let width = extract_number(slide_styles, "width");
    let height = extract_number(slide_styles, "height");
    let margin = extract_number(slide_styles, "margin");

    let layout_rects = slides[idx].layout(
        global,
        if fullscreen {
            Some(Rect {
                x: margin,
                y: margin,
                w: SLIDE_WIDTH - 2 * margin,
                h: SLIDE_HEIGHT - 2 * margin,
            })
        } else {
            None
        },
    );

    SlideData {
        layout_rects,
        background,
        dimensions: (width, height),
    }
}

pub fn generate_textures<'a, T: LoadTexture>(
    global: &GlobalState,
    texture_creator: &'a T,
) -> HashMap<AbstractElementID, Texture<'a>> {
    (0..global.number_of_elements())
        .flat_map(|idx| global.get_element_by_id(AbstractElementID(idx as u32)))
        .filter(|elem| elem.el_type() == ElementType::Image)
        .map(|img| {
            (
                img.id(),
                texture_creator
                    .load_texture(match img.data() {
                        AbstractElementData::Image(path) => path,
                        _ => unreachable!("image element did not have image data"),
                    })
                    .map_err(|err| panic!("{err}"))
                    .unwrap(),
            )
        })
        .collect()
}

pub fn render<T: RenderTarget>(
    global: &GlobalState,
    target: &mut Canvas<T>,
    slide_idx: usize,
    fullscreen: bool,
    texture_map: &HashMap<AbstractElementID, Texture<'_>>,
) {
    let slide_data = generate_slide_data(global, slide_idx, fullscreen);

    target.set_draw_color(slide_data.background);
    target.clear();

    target.set_draw_color((0, 0, 0));
    // target
    //     .fill_rects(
    //         &slide_data
    //             .layout_rects
    //             .iter()
    //             .map(|layout_elem| folium_to_sdl_rect(layout_elem.max_bounds))
    //             .collect::<Vec<_>>(),
    //     )
    //     .unwrap();
    for rect in slide_data.layout_rects {
        let element = global.get_element_by_id(rect.element).unwrap();
        match element.data() {
            AbstractElementData::Row(_) => panic!("Row should never have a layout element of its own"),
            AbstractElementData::Col(_) => panic!("Column should never have a layout element of its own"),
            AbstractElementData::Padding(_) => panic!("Padding should never have a layout element of its own"),
            AbstractElementData::Centre(_) => {}, // TODO
            AbstractElementData::Text(_) => {}, // TODO
            AbstractElementData::Code(_) => {}, // TODO
            AbstractElementData::Image(..) => {
                let texture = texture_map.get(&element.id()).unwrap();
                target.copy(texture, None, folium_to_sdl_rect(rect.max_bounds)).unwrap();
            },
            AbstractElementData::None => {}
        }
    }

    target.present();
}
