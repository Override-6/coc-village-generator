use crate::buidling::{Building, PlotSize};
use crate::cell::Cell;
use crate::label::Bounds;
use crate::render::{get_plate_pixel_position, Image, resize_image_by_width};
use crate::scenery::Scenery;
use ab_glyph::FontRef;
use image::imageops::FilterType;
use image::Rgba;
use imageproc::drawing;

const BUILDING_ALIGNMENT_SHIFT_Y: i64 = 5;

pub(super) fn render_building(
    scenery_image: &mut Image,
    scenery: &Scenery,
    cell: Cell,
    life_points: Option<f32>,
    building: &Building,
) -> Bounds {
    let building_type = &building.building_type;

    let building_image_path = building_type.get_file_path(building.level);
    let building_image = image::open(building_image_path).unwrap();
    let building_image = building_image.to_rgba8();

    let building_size = building_type.self_size();

    let building_size_width = scenery.cell_width() * building_size.cell_diameter() as f32;

    let plot_size = building_type.plot_size();

    let plot_size_width = scenery.cell_width() * plot_size.cell_diameter() as f32;
    let plot_size_height = scenery.cell_height() * plot_size.cell_diameter() as f32;

    let width_margin = if building_size == PlotSize::X1Invisible {
        0f32
    } else {
        scenery.cell_width()
    };

    let target_width = (building_size_width * 2f32 - width_margin) as u32;

    let building_image = resize_image_by_width(&building_image, target_width);

    let (x, y) = get_plate_pixel_position(cell.to_pos(), scenery);

    let plot_pos_x = x;
    let plot_pos_y = y - plot_size_height as i64;

    let plot_center_x = plot_pos_x + plot_size_width as i64;
    let plot_center_y = plot_pos_y + plot_size_height as i64;

    let image_center_x = x + building_image.width() as i64 / 2;
    let image_center_y = y
        + building_image.height() as i64 / 2
        + building_image.height() as i64 / BUILDING_ALIGNMENT_SHIFT_Y;

    let translated_image_x = x + plot_center_x - image_center_x;
    let translated_image_y =
        y + plot_center_y - image_center_y - (scenery.cell_height() / 2f32) as i64;

    image::imageops::overlay(
        scenery_image,
        &building_image,
        translated_image_x,
        translated_image_y,
    );
    #[cfg(debug_assertions)]
    {
        *scenery_image = drawing::draw_cross(
            scenery_image,
            Rgba([255, 0, 0, 255]),
            plot_center_x as i32,
            plot_center_y as i32,
        );
        *scenery_image =
            drawing::draw_cross(scenery_image, Rgba([0, 255, 0, 255]), x as i32, y as i32);
        *scenery_image = drawing::draw_cross(
            scenery_image,
            Rgba([0, 0, 255, 255]),
            image_center_x as i32,
            image_center_y as i32,
        );
        *scenery_image = drawing::draw_cross(
            scenery_image,
            Rgba([0, 255, 255, 255]),
            plot_pos_x as i32,
            plot_pos_y as i32,
        );
    }

    let x_center_pixels = translated_image_x + building_image.width() as i64 / 2;
    let y_center_pixels = translated_image_y + building_image.height() as i64 / 2;

    if let Some(lp) = life_points {
        let font = FontRef::try_from_slice(include_bytes!("../../assets/font.ttf")).unwrap();

        let (color, text) = if lp == 0.0 {
            (Rgba([255, 0, 0, 255]), "DEAD".to_string())
        } else {
            (Rgba([255, 255, 255, 255]), lp.to_string())
        };

        *scenery_image = drawing::draw_text(
            scenery_image,
            color,
            plot_center_x as i32,
            plot_center_y as i32,
            20.0,
            &font,
            &text,
        );
    }

    Bounds {
        x_center: x_center_pixels as f32 / scenery_image.width() as f32,
        y_center: y_center_pixels as f32 / scenery_image.height() as f32,
        height: building_image.height() as f32 / scenery_image.height() as f32,
        width: building_image.width() as f32 / scenery_image.width() as f32,
    }
}

pub(super) fn draw_plot(
    scenery_image: &mut Image,
    scenery: &Scenery,
    plot_size: PlotSize,
    cell: Cell,
) {
    let Some(plot_file) = plot_size.plot_file() else {
        return;
    };

    let plot = image::open(plot_file).unwrap();
    let plot = plot.as_rgba8().unwrap();

    let width_radius = (scenery.cell_width() * plot_size.cell_diameter() as f32) as u32;
    let height_radius = (scenery.cell_height() * plot_size.cell_diameter() as f32) as u32;

    let resized_image = image::imageops::resize(
        plot,
        width_radius * 2,
        height_radius * 2,
        FilterType::Nearest,
    );
    let (x, y) = get_plate_pixel_position(cell.to_pos(), scenery);

    image::imageops::overlay(scenery_image, &resized_image, x, y - height_radius as i64)
}
