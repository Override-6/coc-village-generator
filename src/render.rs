use std::cmp::Ordering;
use std::collections::HashMap;

use image::{GenericImage, GenericImageView, Pixel, Rgb, Rgba};
use image::imageops::FilterType;
use imageproc::drawing;
use imageproc::point::Point;
use lazy_static::lazy_static;
use rand::Rng;

use crate::attack_simulation::{EvolutionLog, TroopId};
use crate::buidling::BUILDING_ASSETS_FOLDER;
use crate::cell::Cell;
use crate::label::Label;
use crate::position::Pos;
use crate::render::building::{draw_plot, render_building};
use crate::render::wall::render_wall;
use crate::scenery::Scenery;
use crate::village::{Component, ComponentType, Village};

mod building;
mod wall;

pub type Image = imageproc::definitions::Image<Rgba<u8>>;

pub struct RenderedScenery {
    pub image: Image,
    pub labels: Vec<Label>,
}

lazy_static! {
    pub static ref BUILDINGS_ASSETS_FILENAMES: Vec<String> =
        std::fs::read_dir(BUILDING_ASSETS_FOLDER)
            .unwrap()
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

pub fn render_logs(
    mut scenery_image: Image,
    scenery: &Scenery,
    logs: Vec<EvolutionLog>,
) -> Result<Image, String> {
    let mut troops_colors: HashMap<TroopId, Rgba<u8>> = HashMap::new();

    for log in logs {
        for (troop, path) in log.troops_paths {
            let color = *troops_colors
                .entry(troop)
                .or_insert_with(|| rand_color().to_rgba());

            if let Some((first, others)) = path.split_first() {
                let (mut last_pixel_x, mut last_pixel_y) =
                    get_plate_pixel_position(*first, scenery);

                for item in others {
                    let (pixel_x, pixel_y) = get_plate_pixel_position(*item, scenery);

                    scenery_image = drawing::draw_line_segment(
                        &scenery_image,
                        (last_pixel_x as f32, last_pixel_y as f32),
                        (pixel_x as f32, pixel_y as f32),
                        color,
                    );

                    last_pixel_x = pixel_x;
                    last_pixel_y = pixel_y;
                }
            }
        }
    }

    Ok(scenery_image)
}

fn rand_color() -> Rgb<u8> {
    let mut rng = rand::thread_rng();
    Rgb([rng.gen(), rng.gen(), rng.gen()])
}

pub fn render(village: &Village) -> Result<RenderedScenery, String> {
    let background_image = image::open("assets/scenery.png").unwrap();
    let mut buffer = background_image.into_rgba8();

    // buffer = draw_debug_grid(&buffer, scenery);

    let scenery = village.scenery();

    let upper_left_corner_cell = Cell {
        x: 0,
        y: scenery.params().plate_height_cells as i16,
    };

    let mut components = village.iter_components().collect::<Vec<_>>();
    components.sort_by(|b1, b2| {
        let building_1_cell = b1.1;
        let building_2_cell = b2.1;

        let distance_b1 = (((upper_left_corner_cell.x - building_1_cell.x) as f32).powi(2)
            + ((upper_left_corner_cell.y - building_1_cell.y) as f32).powi(2))
            .sqrt();
        let distance_b2 = (((upper_left_corner_cell.x - building_2_cell.x) as f32).powi(2)
            + ((upper_left_corner_cell.y - building_2_cell.y) as f32).powi(2))
            .sqrt();

        if distance_b1 < distance_b2 {
            Ordering::Less
        } else if distance_b1 > distance_b2 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });

    for (_, cell, component) in components.clone() {
        if let ComponentType::Building(building) = &component.kind {
            draw_plot(
                &mut buffer,
                scenery,
                building.building_type.plot_size(),
                cell,
            );
        }
    }

    let mut labels = Vec::new();

    for (_, cell, component) in components {
        if let Some(label) = render_component(&mut buffer, village, cell, component) {
            labels.push(label)
        }
    }

    Ok(RenderedScenery {
        image: buffer,
        labels,
    })
}

fn render_component(buffer: &mut Image, village: &Village, cell: Cell, component: &Component) -> Option<Label> {
    match &component.kind {
        ComponentType::Building(building) => {
            let bounds = render_building(buffer, village.scenery(), cell, component.life_points, building);
            let class = BUILDINGS_ASSETS_FILENAMES
                .iter()
                .position(|filename| filename == &building.building_type.get_file_name(building.level))
                .unwrap();
            Some(Label { bounds, class })
        }
        ComponentType::Wall(wall) => {
            render_wall(buffer, village, cell, wall);
            None
        }
    }
}

pub(self) fn resize_image_by_width(image: &Image, target_width: u32) -> Image {
    let width = target_width;
    let height = (image.height() as f32
        * (target_width as f32 / image.width() as f32)) as u32;

    image::imageops::resize(image, width, height, FilterType::Nearest)
}

fn draw_debug_grid(buffer: &Image, scenery: &Scenery) -> Image {
    let bottom_right_corner = scenery.params().bottom_right_corner;
    let bottom_left_corner = scenery.params().bottom_left_corner;
    let upper_right_corner = scenery.params().upper_right_corner;
    let upper_left_corner = scenery.params().upper_left_corner;

    let buffer = drawing::draw_cross(
        buffer,
        Rgb([255, 0, 0]).to_rgba(),
        bottom_right_corner.x,
        bottom_right_corner.y,
    );
    let buffer = drawing::draw_cross(
        &buffer,
        Rgb([0, 255, 0]).to_rgba(),
        bottom_left_corner.x,
        bottom_left_corner.y,
    );
    let buffer = drawing::draw_cross(
        &buffer,
        Rgb([0, 0, 255]).to_rgba(),
        upper_right_corner.x,
        upper_right_corner.y,
    );
    let mut buffer = drawing::draw_cross(
        &buffer,
        Rgb([255, 255, 0]).to_rgba(),
        upper_left_corner.x,
        upper_left_corner.y,
    );

    let colors = [
        Rgba([255, 0, 0, 255]),
        Rgba([0, 255, 0, 255]),
        Rgba([0, 0, 255, 255]),
    ];
    let mut c = 0;

    for x in 0..scenery.params().plate_width_cells as i16 {
        for y in 0..scenery.params().plate_height_cells as i16 {
            buffer = draw_cell(&buffer, scenery, Cell::new(x, y), colors[c]);
            c = (c + 1) % colors.len()
        }
    }

    buffer
}

fn get_plate_pixel_position(pos: Pos, scenery: &Scenery) -> (i64, i64) {
    let left_corner_x = scenery.get_plate_x_axis(
        pos.x * scenery.cell_height(),
        pos.y * scenery.cell_width() - 3f32,
    );
    let left_corner_y = scenery.get_plate_y_axis(
        pos.x * scenery.cell_height(),
        pos.y * scenery.cell_width() - 3f32,
    );

    (left_corner_x as i64, left_corner_y as i64)
}

fn draw_cell<I: GenericImage>(
    image: &I,
    scenery: &Scenery,
    cell: Cell,
    color: I::Pixel,
) -> imageproc::definitions::Image<I::Pixel> {
    let (left_corner_x, left_corner_y) = get_plate_pixel_position(cell.to_pos(), scenery);

    let cell_width = scenery.cell_width();
    let cell_height = scenery.cell_height();

    let poly = [
        Point::new(left_corner_x as i32, left_corner_y as i32),
        Point::new(
            (left_corner_x as f32 + cell_width) as i32,
            (left_corner_y as f32 + cell_height) as i32,
        ),
        Point::new(
            (left_corner_x as f32 + cell_width * 2f32) as i32,
            left_corner_y as i32,
        ),
        Point::new(
            (left_corner_x as f32 + cell_width) as i32,
            (left_corner_y as f32 - cell_height) as i32,
        ),
    ];

    drawing::draw_polygon(image, &poly, color)
}
