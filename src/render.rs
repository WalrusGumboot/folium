use std::collections::HashMap;

use fontdue::{
    layout::{LayoutSettings, TextStyle},
    FontSettings,
};
use itertools::Itertools;
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
    fonts_for_targets: HashMap<(AbstractElementID, StyleTarget), fontdue::Font>,
}

pub struct SlideData {
    layout_rects: Vec<LayoutElement>,
    background: (u8, u8, u8),
    pub dimensions: (u32, u32),
    styles: StyleMap,
    slide_id: AbstractElementID,
}

pub fn generate_slide_data(global: &GlobalState, idx: usize, fullscreen: bool) -> SlideData {
    let slides = global.slides.borrow();
    let all_styles = slides[idx].style_map();
    let slide_styles = all_styles.styles_for_target(&StyleTarget::Slide).unwrap();

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
        slide_id: slides[idx].id(),
    }
}

pub fn initialise_rendering_data<'a, T: LoadTexture>(
    global: &'a GlobalState,
    texture_creator: &'a T,
) -> RenderData<'a> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let fonts_for_targets = (0..global.number_of_slides())
        .flat_map(|slide_idx| {
            let slide = &global.slides.borrow()[slide_idx];
            let fonts_for_slide = global
                .get_slide_elements(slide)
                .iter()
                .filter(|elem| {
                    elem.el_type() == ElementType::Text || elem.el_type() == ElementType::Code
                })
                .map(|elem| match elem.name() {
                    Some(el_name) => StyleTarget::Named(el_name.to_owned()),
                    None => StyleTarget::Anonymous(elem.el_type()),
                })
                .sorted()
                .dedup()
                // .inspect(|st| {
                //     println!("generating font for style target {st:?} on slide {slide_idx}")
                // })
                .map(|st| {
                    let ideal_font_name =
                        extract_string(slide.style_map().styles_for_target(&st).unwrap(), "font");
                    let acquired_font = db.query(&fontdb::Query {
                        families: &[
                            fontdb::Family::Name(&ideal_font_name),
                            fontdb::Family::Serif,
                        ],
                        ..Default::default()
                    });

                    let font_bytes = if let Some(font_id) = acquired_font {
                        match db.face_source(font_id).unwrap().0 {
                            fontdb::Source::Binary(_) => {
                                todo!("cannot handle binary font data loaded into fontdb yet")
                            }
                            fontdb::Source::File(ref path) => {
                                std::fs::read(path).unwrap_or_else(|_| {
                                    panic!(
                                        "got file path {} for font, but could not read it",
                                        path.display()
                                    )
                                })
                            }
                            fontdb::Source::SharedFile(_, _) => {
                                todo!("cannot handle shared files yet")
                            }
                        }
                    } else if cfg!(feature = "builtin-fonts") {
                        eprintln!("warning: specified font '{ideal_font_name}' not found. Use the 'list-fonts' subcommand to see what fonts Folium can use. Falling back to default font");
                        include_bytes!("assets/newsreader.ttf").to_vec()
                    } else {
                        panic!("Specified font '{ideal_font_name}' not found, exiting. Use the 'list-fonts' subcommand to see what fonts Folium can use.")
                    };

                    // SDL2's TTF rendering is pretty horrible and notably quite slow.
                    // We use a fontdue based approach which is much quicker.

                    let font =
                        fontdue::Font::from_bytes(font_bytes, FontSettings::default()).unwrap();

                    ((slide.id(), st), font)
                })
                .collect_vec();

            fonts_for_slide
        })
        .collect::<HashMap<(AbstractElementID, StyleTarget), fontdue::Font>>();

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
            .inspect(|(id, tex)| println!("{id} has texture {:?}", tex.query()))
            .collect(),
        font_database: db,
        fonts_for_targets,
    }
}

pub fn render<T: RenderTarget>(
    global: &GlobalState,
    target: &mut Canvas<T>,
    slide_idx: usize,
    fullscreen: bool,
    render_data: &RenderData,
    debug_rects: bool,
) {
    let slide_data = generate_slide_data(global, slide_idx, fullscreen);

    target.set_draw_color(slide_data.background);
    target.clear();

    if debug_rects {
        target.set_draw_color((255, 0, 0));
        target
            .draw_rects(
                &slide_data
                    .layout_rects
                    .iter()
                    .map(|r| folium_to_sdl_rect(r.max_bounds))
                    .collect::<Vec<_>>(),
            )
            .unwrap();
    }

    for rect in slide_data.layout_rects {
        let element = global.get_element_by_id(rect.element).unwrap();
        match element.data() {
            AbstractElementData::Sized(_) => {
                panic!("Sized should never have a layout element of its own")
            }
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
                let text_style_target = StyleTarget::reify(&element);

                let text_style = slide_data
                    .styles
                    .styles_for_target(&text_style_target)
                    .unwrap();

                target.set_blend_mode(sdl2::render::BlendMode::Blend);

                let font = render_data
                    .fonts_for_targets
                    .get(&(slide_data.slide_id, text_style_target))
                    .unwrap();
                let font_size = extract_number(text_style, "size") as f32;
                let text_colour = extract_colour(text_style, "fill");

                let mut layout =
                    fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
                layout.reset(&LayoutSettings {
                    x: 0.0,
                    y: 0.0,
                    max_width: Some(rect.max_bounds.w as f32),
                    max_height: Some(rect.max_bounds.h as f32),
                    ..Default::default()
                });
                layout.append(
                    &[font],
                    &TextStyle::new(text_to_be_rendered, font_size, 0),
                );
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
                            target
                                .draw_point((
                                    glyph.x as i32 + x_off as i32 + rect.max_bounds.x as i32,
                                    glyph.y as i32 + y_off as i32 + rect.max_bounds.y as i32,
                                ))
                                .unwrap();
                        }
                    }
                }
            }
            AbstractElementData::Code(code_to_be_rendered) => {
                let code_style_target = StyleTarget::reify(&element);

                let code_style = slide_data
                    .styles
                    .styles_for_target(&code_style_target)
                    .unwrap();

                let bg_colour = extract_colour(code_style, "bg");

                target.set_draw_color(bg_colour);
                target
                    .fill_rect(folium_to_sdl_rect(rect.max_bounds))
                    .unwrap();

                let font = render_data
                    .fonts_for_targets
                    .get(&(slide_data.slide_id, code_style_target))
                    .unwrap();

                let font_size = extract_number(code_style, "size") as f32;
                let text_colour = extract_colour(code_style, "fill");

                let box_margin = extract_number(code_style, "margin");
                let text_area = rect.max_bounds.with_margin(box_margin);

                let mut layout =
                    fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
                layout.reset(&LayoutSettings {
                    y: 0.0,
                    x: 0.0,
                    max_width: Some(text_area.w as f32),
                    max_height: Some(text_area.h as f32),
                    ..Default::default()
                });
                layout.append(
                    &[font],
                    &TextStyle::new(code_to_be_rendered, font_size, 0),
                );
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
                            target
                                .draw_point((
                                    glyph.x as i32 + x_off as i32 + text_area.x as i32,
                                    glyph.y as i32 + y_off as i32 + text_area.y as i32,
                                ))
                                .unwrap();
                        }
                    }
                }
            } // TODO: add code-specific features, like syntax highlighting etc
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
