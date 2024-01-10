use sdl2::render::{Canvas, RenderTarget};

use crate::{
    ast::GlobalState,
    layout::LayoutElement,
    style::{extract_colour, StyleTarget},
};

pub struct SlideData {
    layout_rects: Vec<LayoutElement>,
    background: (u8, u8, u8),
}

fn generate_slide_data(global: &GlobalState, idx: usize) -> SlideData {
    let slides = global.slides.borrow();

    let layout_rects = slides[idx].layout(&global, None);
    let background = extract_colour(
        slides[idx]
            .style_map()
            .styles_for_target(StyleTarget::Slide)
            .unwrap(),
        "bg",
    );

    SlideData {
        layout_rects,
        background,
    }
}

pub fn render<T: RenderTarget>(global: &GlobalState, target: &mut Canvas<T>, slide_idx: usize) {
    let slide_data = generate_slide_data(global, slide_idx);

    target.set_draw_color(slide_data.background);
    target.clear();

    target.set_draw_color((0, 0, 0));
    target
        .fill_rects(
            &slide_data
                .layout_rects
                .iter()
                .map(|layout_elem| {
                    let folium_rect = layout_elem.max_bounds;
                    sdl2::rect::Rect::new(
                        folium_rect.x as i32,
                        folium_rect.y as i32,
                        folium_rect.w,
                        folium_rect.h,
                    )
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

    target.present();
}
