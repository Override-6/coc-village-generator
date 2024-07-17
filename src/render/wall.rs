use image::GenericImageView;
use crate::cell::Cell;
use crate::label::Bounds;
use crate::render::{get_plate_pixel_position, Image, resize_image_by_width};
use crate::village::Village;
use crate::wall::{Wall, WallConnectionType};

pub(super) fn render_wall(
    scenery_image: &mut Image,
    village: &Village,
    cell: Cell,
    wall: &Wall,
) -> Bounds {
    let wall_state = village.get_wall_connection_type(cell).expect("given cell does not hosts any wall");
    let state_name = wall_state.name();

    let scenery = village.scenery();

    let wall_image_file = format!("assets/walls/level_{}/{state_name}.png", wall.level);

    let wall_image = image::open(wall_image_file).unwrap().to_rgba8();
    let wall_image = resize_image_by_width(&wall_image, (wall_image.width() as f32 * wall_state.size_ratio()) as u32);

    let (x, y) = get_plate_pixel_position(cell.to_pos(), scenery);
    let height_radius = wall_image.height() as i64;
    let width_radius = wall_image.width() as i64;

    let x = x + (height_radius as f32 * wall_state.width_shift_ratio()) as i64;
    let y = y - (width_radius as f32 * wall_state.height_shift_ratio()) as i64;

    image::imageops::overlay(scenery_image, &wall_image, x, y);

    let x_center_pixels = x as f32 + scenery.cell_width() / 2.0;
    let y_center_pixels = y as f32 + scenery.cell_height() / 2.0;

    Bounds {
        x_center: x_center_pixels / scenery_image.width() as f32,
        y_center: y_center_pixels / scenery_image.height() as f32,
        height: wall_image.height() as f32 / scenery_image.height() as f32,
        width: wall_image.width() as f32 / scenery_image.width() as f32,
    }
}
