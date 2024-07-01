use std::cmp::Ordering;

use image::{GenericImage, GenericImageView, Pixel, Rgb, Rgba};
use image::imageops::FilterType;
use imageproc::drawing;
use imageproc::point::Point;
use lazy_static::lazy_static;

use crate::buidling::{Building, BUILDING_ASSETS_FOLDER, PlotSize};
use crate::cell::Cell;
use crate::label::{Bounds, Label};
use crate::scenery::Scenery;
use crate::village::Village;

type Image = imageproc::definitions::Image<Rgba<u8>>;

pub struct RenderedScenery {
    pub image: Image,
    pub labels: Vec<Label>,
}

lazy_static! {
    pub static ref BUILDINGS_ASSETS_FILENAMES: Vec<String> = std::fs::read_dir(BUILDING_ASSETS_FOLDER)
        .unwrap()
        .into_iter()
        .map(|file| file
            .unwrap()
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string())
        .collect();
}

pub fn render(scenery: &Scenery, village: &Village) -> Result<RenderedScenery, String> {
    let background_image = image::open("assets/scenery.png").unwrap();
    let mut buffer = background_image.into_rgba8();

    let upper_left_corner_cell = Cell { x: 0, y: scenery.params().plate_height_cells as i16 };

    let mut buildings = village.iter_buildings().collect::<Vec<_>>();
    buildings.sort_by(|b1, b2| {
        let b1 = b1.pos;
        let b2 = b2.pos;

        let distance_b1 = (((upper_left_corner_cell.x - b1.x) as f32).powi(2) + ((upper_left_corner_cell.y - b1.y) as f32).powi(2)).sqrt();
        let distance_b2 = (((upper_left_corner_cell.x - b2.x) as f32).powi(2) + ((upper_left_corner_cell.y - b2.y) as f32).powi(2)).sqrt();

        if distance_b1 < distance_b2 {
            Ordering::Less
        } else if distance_b1 > distance_b2 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });

    for building in buildings.clone() {
        draw_plot(&mut buffer, scenery, building.building_type.plot_size(), building.pos);
    }

    let mut labels = Vec::new();


    for building in buildings {
        let bounds = render_building_image(&mut buffer, scenery, building);
        let class = BUILDINGS_ASSETS_FILENAMES.iter().position(|filename| filename == &building.building_type.get_file_name(building.level)).unwrap();

        labels.push(Label {
            bounds,
            class,
        })
    }

    Ok(RenderedScenery {
        image: buffer,
        labels,
    })
}

const BUILDING_ALIGNMENT_SHIFT_Y: i64 = 5;

fn render_building_image(scenery_image: &mut Image, scenery: &Scenery, building: &Building) -> Bounds {
    let building_type = &building.building_type;
    let cell = building.pos;

    let building_image_path = building_type.get_file_path(building.level);
    let building_image = image::open(building_image_path).unwrap();
    let building_image = building_image.as_rgba8().unwrap();

    let building_size = building_type.self_size();

    let building_size_width = scenery.cell_width_radius() * building_size.cell_diameter() as f32;

    let plot_size = building_type.plot_size();

    let plot_size_width = scenery.cell_width_radius() * plot_size.cell_diameter() as f32;
    let plot_size_height = scenery.cell_height_radius() * plot_size.cell_diameter() as f32;

    let width_margin = if building_size == PlotSize::X1Invisible { 0f32 } else { scenery.cell_width_radius() };

    let target_width = (building_size_width * 2f32 - width_margin) as u32;

    let width = target_width;
    let height = (building_image.height() as f32 * (target_width as f32 / building_image.width() as f32)) as u32;

    let building_image = image::imageops::resize(
        building_image,
        width,
        height,
        FilterType::Nearest,
    );

    let (x, y) = get_plate_pixel_position(cell, scenery);

    let plot_pos_x = x;
    let plot_pos_y = y - plot_size_height as i64;

    let plot_center_x = plot_pos_x + plot_size_width as i64;
    let plot_center_y = plot_pos_y + plot_size_height as i64;

    let image_center_x = x + building_image.width() as i64 / 2;
    let image_center_y = y + building_image.height() as i64 / 2 + building_image.height() as i64 / BUILDING_ALIGNMENT_SHIFT_Y;

    let translated_image_x = x + plot_center_x - image_center_x;
    let translated_image_y = y + plot_center_y - image_center_y - (scenery.cell_height_radius() / 2f32) as i64;

    image::imageops::overlay(scenery_image, &building_image, translated_image_x, translated_image_y);
    #[cfg(debug_assertions)] {
        *scenery_image = drawing::draw_cross(scenery_image, Rgba([255, 0, 0, 255]), plot_center_x as i32, plot_center_y as i32);
        *scenery_image = drawing::draw_cross(scenery_image, Rgba([0, 255, 0, 255]), x as i32, y as i32);
        *scenery_image = drawing::draw_cross(scenery_image, Rgba([0, 0, 255, 255]), image_center_x as i32, image_center_y as i32);
        *scenery_image = drawing::draw_cross(scenery_image, Rgba([0, 255, 255, 255]), plot_pos_x as i32, plot_pos_y as i32);
    }

    let x_center_pixels = translated_image_x + building_image.width() as i64 / 2;
    let y_center_pixels = translated_image_y + building_image.height() as i64 / 2;

    Bounds {
        x_center: x_center_pixels as f32 / scenery_image.width() as f32,
        y_center: y_center_pixels as f32 / scenery_image.height() as f32,
        height: building_image.height() as f32 / scenery_image.height() as f32,
        width: building_image.width() as f32 / scenery_image.width() as f32,
    }
}


fn draw_plot(scenery_image: &mut Image, scenery: &Scenery, plot_size: PlotSize, cell: Cell) {
    let Some(plot_file) = plot_size.plot_file() else { return };

    let plot = image::open(plot_file).unwrap();
    let plot = plot.as_rgba8().unwrap();

    let width_radius = (scenery.cell_width_radius() * plot_size.cell_diameter() as f32) as u32;
    let height_radius = (scenery.cell_height_radius() * plot_size.cell_diameter() as f32) as u32;

    let resized_image = image::imageops::resize(
        plot,
        width_radius * 2,
        height_radius * 2,
        FilterType::Nearest,
    );
    let (x, y) = get_plate_pixel_position(cell, scenery);

    image::imageops::overlay(scenery_image, &resized_image, x, y - height_radius as i64)
}


fn draw_debug_grid(buffer: &Image, scenery: &Scenery) -> Image {
    let bottom_right_corner = scenery.params().bottom_right_corner;
    let bottom_left_corner = scenery.params().bottom_left_corner;
    let upper_right_corner = scenery.params().upper_right_corner;
    let upper_left_corner = scenery.params().upper_left_corner;

    let buffer = drawing::draw_cross(buffer, Rgb([255, 0, 0]).to_rgba(), bottom_right_corner.x, bottom_right_corner.y);
    let buffer = drawing::draw_cross(&buffer, Rgb([0, 255, 0]).to_rgba(), bottom_left_corner.x, bottom_left_corner.y);
    let buffer = drawing::draw_cross(&buffer, Rgb([0, 0, 255]).to_rgba(), upper_right_corner.x, upper_right_corner.y);
    let mut buffer = drawing::draw_cross(&buffer, Rgb([255, 255, 0]).to_rgba(), upper_left_corner.x, upper_left_corner.y);

    let colors = [Rgba([255, 0, 0, 255]), Rgba([0, 255, 0, 255]), Rgba([0, 0, 255, 255])];
    let mut c = 0;

    for x in 0..scenery.params().plate_width_cells as i16 {
        for y in 0..scenery.params().plate_height_cells as i16 {
            buffer = draw_cell(&buffer, scenery, Cell::new(x, y), colors[c]);
            c = (c + 1) % colors.len()
        }
    }

    buffer
}

fn get_plate_pixel_position(cell: Cell, scenery: &Scenery) -> (i64, i64) {
    let left_corner_x = scenery.get_plate_x_axis(cell.x as f32 * scenery.cell_height_radius(), cell.y as f32 * scenery.cell_width_radius());
    let left_corner_y = scenery.get_plate_y_axis(cell.x as f32 * scenery.cell_height_radius(), cell.y as f32 * scenery.cell_width_radius());

    (left_corner_x as i64, left_corner_y as i64)
}

fn draw_cell<I: GenericImage>(image: &I, scenery: &Scenery, cell: Cell, color: I::Pixel) -> imageproc::definitions::Image<I::Pixel> {
    let (left_corner_x, left_corner_y) = get_plate_pixel_position(cell, scenery);

    let cell_width = scenery.cell_width_radius();
    let cell_height = scenery.cell_height_radius();

    let poly = [
        Point::new(left_corner_x as i32, left_corner_y as i32),
        Point::new((left_corner_x as f32 + cell_width) as i32, (left_corner_y as f32 + cell_height) as i32),
        Point::new((left_corner_x as f32 + cell_width * 2f32) as i32, left_corner_y as i32),
        Point::new((left_corner_x as f32 + cell_width) as i32, (left_corner_y as f32 - cell_height) as i32)
    ];

    drawing::draw_polygon(image, &poly, color)
}

