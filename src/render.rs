use std::collections::HashMap;

use fontdue::{
    layout::{LayoutSettings, TextStyle},
    FontSettings,
};
use sdl2::{
    image::LoadTexture,
    render::{Canvas, RenderTarget, Texture},
};

use crate::{
    ast::{AbstractElementData, AbstractElementID, ElementType, GlobalState},
    layout::{folium_to_sdl_rect, LayoutElement, Rect},
    style::{extract_colour, extract_number, extract_string, StyleMap, StyleTarget},
    SLIDE_HEIGHT, SLIDE_WIDTH,
};

pub struct RenderData<'a> {
    texture_map: HashMap<AbstractElementID, Texture<'a>>,
    font_database: fontdb::Database,
}

pub struct SlideData {
    layout_rects: Vec<LayoutElement>,
    background: (u8, u8, u8),
    pub dimensions: (u32, u32),
    styles: StyleMap,
}

pub fn generate_slide_data(global: &GlobalState, idx: usize, fullscreen: bool) -> SlideData {
    let slides = global.slides.borrow();
    let all_styles = slides[idx].style_map();
    let slide_styles = all_styles.styles_for_target(StyleTarget::Slide).unwrap();

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
        styles: all_styles.clone(), // TODO: don't clone here
    }
}

pub fn initialise_rendering_data<'a, T: LoadTexture>(
    global: &'a GlobalState,
    texture_creator: &'a T,
) -> RenderData<'a> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    RenderData {
        texture_map: (0..global.number_of_elements())
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
            .collect(),
        font_database: db,
    }
}

pub fn render<T: RenderTarget>(
    global: &GlobalState,
    target: &mut Canvas<T>,
    slide_idx: usize,
    fullscreen: bool,
    render_data: &RenderData,
) {
    let slide_data = generate_slide_data(global, slide_idx, fullscreen);

    target.set_draw_color(slide_data.background);
    target.clear();

    // target.set_draw_color((0, 0, 0));
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
            AbstractElementData::Row(_) => {
                panic!("Row should never have a layout element of its own")
            }
            AbstractElementData::Col(_) => {
                panic!("Column should never have a layout element of its own")
            }
            AbstractElementData::Padding(_) => {
                panic!("Padding should never have a layout element of its own")
            }
            AbstractElementData::Centre(_) => {} // TODO
            AbstractElementData::Text(text_to_be_rendered) => {
                // TODO: this will only work properly when named styles work;
                // for now, this will crash a lot because named styles do not
                // fill in implicit style specifications
                let text_style = match element.name() {
                    Some(el_name) => slide_data
                        .styles
                        .styles_for_target(StyleTarget::Named(el_name.to_owned())),
                    None => slide_data
                        .styles
                        .styles_for_target(StyleTarget::Anonymous(ElementType::Text)),
                }
                .unwrap();

                let ideal_font_name = extract_string(text_style, "font");
                let acquired_font = render_data.font_database.query(&fontdb::Query {
                    families: &[
                        fontdb::Family::Name(&ideal_font_name),
                        fontdb::Family::Serif,
                    ],
                    ..Default::default()
                });

                let font_bytes = if let Some(font_id) = acquired_font {
                    match render_data.font_database.face_source(font_id).unwrap().0 {
                        fontdb::Source::Binary(_) => {
                            todo!("cannot handle binary font data loaded into fontdb yet")
                        }
                        fontdb::Source::File(ref path) => std::fs::read(path).expect(&format!(
                            "got file path {} for font, but could not read it",
                            path.display()
                        )),
                        fontdb::Source::SharedFile(_, _) => todo!("cannot handle shared files yet"),
                    }
                } else {
                    include_bytes!("assets/newsreader.ttf").to_vec()
                };

                // SDL2's TTF rendering is pretty horrible and notably quite slow.
                // We use a fontdue based approach which is much quicker.

                target.set_blend_mode(sdl2::render::BlendMode::Blend);

                let font = fontdue::Font::from_bytes(font_bytes, FontSettings::default()).unwrap();
                let font_size = extract_number(text_style, "size") as f32;
                let text_colour = extract_colour(text_style, "fill");

                let mut layout =
                    fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
                layout.reset(&LayoutSettings {
                    x: 0.0,
                    y: 0.0,
                    ..Default::default()
                });
                layout.append(
                    &[&font],
                    &TextStyle::new(&text_to_be_rendered, font_size, 0),
                );

                // println!(
                //     "rendering \"{text_to_be_rendered}\" to position ({}, {})",
                //     rect.max_bounds.x, rect.max_bounds.y
                // );

                // let tick = std::time::Instant::now();
                // println!("started rendering glyphs");
                for glyph in layout.glyphs() {
                    let (_, coverage) = font.rasterize(glyph.parent, font_size);

                    for y_off in 0..glyph.height {
                        for x_off in 0..glyph.width {
                            let cov = coverage[y_off * glyph.width + x_off];
                            target.set_draw_color(sdl2::pixels::Color::RGBA(
                                text_colour.0,
                                text_colour.1,
                                text_colour.2,
                                cov,
                            ));
                            // println!(
                            //     "printed {:?} to position ({}, {})",
                            //     target.draw_color().rgba(),
                            //     glyph.x as i32 + x_off as i32 + rect.max_bounds.x as i32,
                            //     glyph.y as i32 + y_off as i32 + rect.max_bounds.y as i32,
                            // );
                            target
                                .draw_point((
                                    glyph.x as i32 + x_off as i32 + rect.max_bounds.x as i32,
                                    glyph.y as i32 + y_off as i32 + rect.max_bounds.y as i32,
                                ))
                                .unwrap();

                        }
                    }
                }
                // let tock = std::time::Instant::now();
                // println!(
                //     "finished rendering glyphs, took {} μs",
                //     (tock - tick).as_micros()
                // );
            }
            AbstractElementData::Code(_) => {} // TODO
            AbstractElementData::Image(..) => {
                let texture = render_data.texture_map.get(&element.id()).unwrap();
                target
                    .copy(texture, None, folium_to_sdl_rect(rect.max_bounds))
                    .unwrap();
            }
            AbstractElementData::None => {}
        }
    }

    target.present();
}
