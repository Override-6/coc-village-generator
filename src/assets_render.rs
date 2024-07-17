use image::imageops::FilterType;
use image::RgbaImage;
use rand::Rng;

use crate::label::{Bounds, Label};
use crate::render::RenderedScenery;
use crate::scenery::Scenery;

pub struct Asset {
    pub path: String,
    pub class: usize,
}

pub fn render(img: RgbaImage, assets: &[&Asset]) -> Result<RenderedScenery, String> {
    let mut rng = rand::thread_rng();

    let mut buff = img;

    let mut labels = Vec::new();

    for asset in assets {
        let asset_img = image::open(&asset.path).unwrap();

        let x = rng.gen_range(0f32..buff.width() as f32 - asset_img.width() as f32) as i64;
        let y = rng.gen_range(0f32..buff.height() as f32 - asset_img.height() as f32) as i64;

        let size_ratio = rng.gen_range(0.6..1.0);

        let resized_image = image::imageops::resize(
            &asset_img,
            (asset_img.width() as f32 * size_ratio) as u32,
            (asset_img.height() as f32 * size_ratio) as u32,
            FilterType::Nearest,
        );

        image::imageops::overlay(&mut buff, &resized_image, x, y);

        let x_center = x + resized_image.width() as i64 / 2;
        let y_center = y + resized_image.height() as i64 / 2;

        let bounds = Bounds {
            x_center: x_center as f32 / buff.width() as f32,
            y_center: y_center as f32 / buff.height() as f32,
            height: resized_image.height() as f32 / buff.height() as f32,
            width: resized_image.width() as f32 / buff.width() as f32,
        };

        labels.push(Label {
            bounds,
            class: asset.class,
        })
    }

    Ok(RenderedScenery {
        image: buff,
        labels,
    })
}

fn get_plate_pixel_position(x: f32, y: f32, scenery: &Scenery) -> (i64, i64) {
    let left_corner_x =
        scenery.get_plate_x_axis(x * scenery.cell_height(), y * scenery.cell_width());
    let left_corner_y =
        scenery.get_plate_y_axis(x * scenery.cell_height(), y * scenery.cell_width());

    (left_corner_x as i64, left_corner_y as i64)
}
