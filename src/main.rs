use std::fs::OpenOptions;
use image::{GenericImage, Pixel};
use imageproc::drawing::Canvas;
use rand::distributions::Distribution;
use rand::Rng;
use tokio::task::JoinSet;
use crate::buidling::{ArcherDefenceState, Building, BUILDING_ASSETS_FOLDER, BuildingType, MissileDefenceState};
use crate::cell::Cell;
use crate::render::{BUILDINGS_ASSETS_FILENAMES, render};
use crate::scenery::{default_scenery, Scenery};
use crate::village::{Village, VillageOperationError, VillageOperationResult};
use std::io::Write;
use crate::label::Bounds;

mod buidling;
mod cell;
mod scenery;
mod village;
mod render;
mod label;

const IMAGE_COUNT: usize = 1000;

#[tokio::main(flavor = "multi_thread", worker_threads = 20)]
async fn main() {
    let mut set = JoinSet::new();

    for x in 0..(IMAGE_COUNT as f32 * 0.8f32) as usize {
        set.spawn(generate_village_image("out/images/train", x));
    }

    for x in 0..(IMAGE_COUNT as f32 * 0.2f32) as usize {
        set.spawn(generate_village_image("out/images/val", x));
    }

    let mut progress = 0;

    while let Some(res) = set.join_next().await {
        res.unwrap();
        print!("progress : {progress}/{IMAGE_COUNT}\r");
        progress += 1;
    }

    println!("Generating YOLO dataset configuration");
    let mut dataset_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("out/data.yaml")
        .expect("cannot open file");

    writeln!(dataset_file, "train: ./images/train").unwrap();
    writeln!(dataset_file, "val: ./images/val").unwrap();
    writeln!(dataset_file, "nc: {}", BUILDINGS_ASSETS_FILENAMES.len()).unwrap();
    write!(dataset_file, "classes: [").unwrap();

    if let Some((first, other)) = BUILDINGS_ASSETS_FILENAMES.split_first() {
        write!(dataset_file, "{first}").unwrap();
        for it in other {
            write!(dataset_file, ", {it}").unwrap();
        }
    }
    writeln!(dataset_file, "]").unwrap();
}

async fn generate_village_image(dir: &str, id: usize) {
    let scenery = default_scenery();

    let village = generate_village(&scenery).unwrap();
    // let village = create_village().unwrap();

    let result = render(&scenery, &village).unwrap();

    result.image.save(format!("{dir}/village_{id}.png")).unwrap();

    let mut labels_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(format!("{dir}/village_{id}.txt"))
        .expect("cannot open file");

    for label in result.labels {
        let Bounds {
            x_center,
            y_center,
            width,
            height
        } = label.bounds;
        writeln!(labels_file, "{} {x_center} {y_center} {width} {height}", label.class).unwrap();
    }
}

fn generate_village(scenery: &Scenery) -> VillageOperationResult<Village> {
    let mut rng = rand::thread_rng();

    let mut village = Village::default();

    for x in 0..scenery.params().plate_width_cells as i16 {
        for y in 0..scenery.params().plate_height_cells as i16 {
            let building_type: BuildingType = rng.gen();
            let level = rng.gen_range(building_type.level_range());
            let pos = Cell::new(x, y);

            let building = Building {
                building_type,
                level,
                pos,
            };

            match village.add_building(building) {
                Ok(()) => {}
                Err(VillageOperationError::BuildingCollides) => {} //do not panic if the building collides with another one
                other => other.unwrap() //force panic
            };
        }
    }

    Ok(village)
}


fn create_village() -> VillageOperationResult<Village> {
    let mut village = Village::default();

    village.add_building(Building {
        building_type: BuildingType::ArmyCamp,
        pos: Cell::new(20, 15),
        level: 3,
    })?;

    village.add_building(Building {
        building_type: BuildingType::Mortar(MissileDefenceState::Regular),
        pos: Cell::new(3, 6),
        level: 13,
    })?;

    village.add_building(Building {
        building_type: BuildingType::BuilderHut,
        pos: Cell::new(0, 0),
        level: 1,
    })?;

    village.add_building(Building {
        building_type: BuildingType::ArcherTower(ArcherDefenceState::Regular),
        pos: Cell::new(8, 15),
        level: 12,
    })?;

    village.add_building(Building {
        building_type: BuildingType::HiddenTesla,
        pos: Cell::new(7, 12),
        level: 9,
    })?;

    village.add_building(Building {
        building_type: BuildingType::HiddenTesla,
        pos: Cell::new(43, 43),
        level: 9,
    })?;

    village.add_building(Building {
        building_type: BuildingType::HiddenTesla,
        pos: Cell::new(3, 3),
        level: 9,
    })?;

    village.add_building(Building {
        building_type: BuildingType::TownHall,
        pos: Cell::new(20, 35),
        level: 11,
    })?;

    Ok(village)
}

