use std::env::current_dir;
use std::fs::OpenOptions;
use std::future::Future;
use std::io::Write;

use image::Rgba;
use lazy_static::lazy_static;
use rand::Rng;
use tokio::task::JoinSet;

use crate::assets_render::Asset;
use crate::attack_simulation::AttackPlan;
use crate::buidling::{
    ArcherDefenceState, Building, BuildingCharacteristics, BuildingType, MissileDefenceState,
};
use crate::cell::Cell;
use crate::label::Bounds;
use crate::position::Pos;
use crate::render::{BUILDINGS_ASSETS_FILENAMES, Image, render, render_logs, RenderedScenery};
use crate::scenery::Scenery;
use crate::troop::{Troop, TroopType};
use crate::village::{Component, ComponentType, Village, VillageOperationError, VillageOperationResult};
use crate::wall::Wall;

mod assets_render;
mod attack_simulation;
mod buidling;
mod cell;
mod label;
mod pathfinding;
mod position;
mod render;
mod scenery;
mod troop;
mod village;
mod wall;

const IMAGE_COUNT: usize = 20000;

#[tokio::main(flavor = "multi_thread", worker_threads = 20)]
async fn main() {
    // village_generation().await;
    // assets_mess_generation().await;
    village_attack_simulation().await;
}

async fn village_attack_simulation() {
    let village = create_village().unwrap();

    let attack_plan = AttackPlan {
        initial_placements: vec![
            Troop {
                tpe: TroopType::Barbarian,
                pos: Pos::new(21.0, 21.0),
            },
            Troop {
                tpe: TroopType::Barbarian,
                pos: Pos::new(0.0, 21.0),
            },
            Troop {
                tpe: TroopType::Giant,
                pos: Pos::new(0.0, 0.0),
            },
            Troop {
                tpe: TroopType::Giant,
                pos: Pos::new(44.0, 21.0),
            },
            Troop {
                tpe: TroopType::Barbarian,
                pos: Pos::new(44.0, 44.0),
            },
        ],
    };

    // let simulation_result =
    //     attack_simulation::simulate_attack(63, &village, &attack_plan);

    let mut render_result = render(&village).unwrap();

    // render_result.image = render_logs(
    //     render_result.image,
    //     village.scenery(),
    //     simulation_result.evolution_logs,
    // )
    //     .unwrap();

    render_result
        .image
        .save("out/simulations/simulation.png")
        .unwrap();
}

lazy_static! {
    static ref ASSETS: Vec<Asset> = {
        let assets_folders = ["assets/sprites/buildings/"];

        let mut assets = Vec::new();

        print!("{}", current_dir().unwrap().display());

        for folder in assets_folders {
            for file in std::fs::read_dir(folder).unwrap() {
                let file = file.unwrap();

                let path = format!("{}", file.path().display());
                assets.push(Asset {
                    path,
                    class: assets.len(),
                });
            }
        }
        assets
    };
}

async fn assets_mess_generation() {
    let assets_folders = ["assets/sprites/buildings"];

    let mut assets = Vec::new();

    for folder in assets_folders {
        for file in std::fs::read_dir(folder).unwrap() {
            let path = format!("{}", file.unwrap().path().display());
            assets.push(Asset {
                path,
                class: assets.len(),
            });
        }
    }

    generate_dataset(
        ASSETS.len(),
        ASSETS.iter().map(|a| &a.path),
        generate_asset_mess_image,
    )
        .await;
}

async fn generate_asset_mess_image(image_dir: &str, labels_dir: &str, id: usize) {
    const ASSETS_PER_IMAGE: u16 = 75;

    let mut used_assets = Vec::new();

    for _ in 0..ASSETS_PER_IMAGE {
        let mut rng = rand::thread_rng();

        used_assets.push(&ASSETS[rng.gen_range(0..ASSETS.len())])
    }

    let grid = image::open("assets/grid.png").unwrap();
    let result = assets_render::render(grid.to_rgba8(), &used_assets).unwrap();

    generate_dataset_entries(result, image_dir, labels_dir, id).await;
}

async fn village_generation() {
    generate_dataset(
        BUILDINGS_ASSETS_FILENAMES.len(),
        BUILDINGS_ASSETS_FILENAMES.iter(),
        generate_village_image,
    )
        .await;
}

async fn generate_dataset<'a, F>(
    asset_count: usize,
    assets: impl Iterator<Item=&String>,
    generate_entries: impl Fn(&'a str, &'a str, usize) -> F + 'a,
) where
    F: Future<Output=()>,
    F: Send + 'static,
{
    std::fs::create_dir_all("out/images/train").unwrap();
    std::fs::create_dir_all("out/images/val").unwrap();
    std::fs::create_dir_all("out/images/test").unwrap();
    std::fs::create_dir_all("out/labels/train").unwrap();
    std::fs::create_dir_all("out/labels/val").unwrap();
    std::fs::create_dir_all("out/labels/test").unwrap();

    let mut set = JoinSet::new();

    for x in 0..(IMAGE_COUNT as f32 * 0.8f32) as usize {
        set.spawn(generate_entries("out/images/train", "out/labels/train", x));
    }

    for x in 0..(IMAGE_COUNT as f32 * 0.2f32) as usize {
        set.spawn(generate_entries("out/images/val", "out/labels/val", x));
    }

    for x in 0..(IMAGE_COUNT as f32 * 0.2f32) as usize {
        set.spawn(generate_entries("out/images/test", "out/labels/test", x));
    }

    let mut progress = 0;
    let total = set.len();

    while let Some(res) = set.join_next().await {
        res.unwrap();

        print!("progress : {progress}/{total}\r");
        progress += 1;
    }

    println!("Generating YOLO dataset configuration");
    let mut dataset_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("out/data.yaml")
        .expect("cannot open file");

    writeln!(
        dataset_file,
        "train: /home/maxime/Projects/coc-base-generator/out/images/train"
    )
        .unwrap();
    writeln!(
        dataset_file,
        "val: /home/maxime/Projects/coc-base-generator/out/images/val"
    )
        .unwrap();
    writeln!(dataset_file, "nc: {}", asset_count).unwrap();
    write!(dataset_file, "classes: [").unwrap();

    if let Some((first, other)) = assets.collect::<Vec<_>>().split_first() {
        write!(dataset_file, "{first}").unwrap();
        for it in other {
            write!(dataset_file, ", {it}").unwrap();
        }
    }
    writeln!(dataset_file, "]").unwrap();
}

async fn generate_village_image(image_dir: &str, labels_dir: &str, id: usize) {
    // let village = generate_village(&scenery).unwrap();
    let village = create_village().unwrap();
    //
    // let village = Village::default();

    let result = render(&village).unwrap();

    generate_dataset_entries(result, image_dir, labels_dir, id).await;
}

async fn generate_dataset_entries(
    mut result: RenderedScenery,
    image_dir: &str,
    labels_dir: &str,
    id: usize,
) {
    result
        .image
        .save(format!("{image_dir}/village_{id}.png"))
        .unwrap();

    result.image = add_random_dots(result.image, 450, 2, 5);
    result
        .image
        .save(format!("{image_dir}/village_{id}_with_dots.png"))
        .unwrap();

    let mut labels_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(format!("{labels_dir}/village_{id}.txt"))
        .expect("cannot open file");

    for label in result.labels {
        let Bounds {
            x_center,
            y_center,
            width,
            height,
        } = label.bounds;
        writeln!(
            labels_file,
            "{} {x_center} {y_center} {width} {height}",
            label.class
        )
            .unwrap();
    }

    std::fs::copy(
        format!("{labels_dir}/village_{id}.txt"),
        format!("{labels_dir}/village_{id}_with_dots.txt"),
    )
        .unwrap();
}

fn add_random_dots(mut image: Image, dots_count: u16, min_size: u32, max_size: u32) -> Image {
    let mut rng = rand::thread_rng();
    for _ in 0..dots_count {
        let radius = rng.gen_range(min_size..=max_size);
        let x = rng.gen_range(0..image.width() - radius) as i32;
        let y = rng.gen_range(0..image.height() - radius) as i32;

        image = imageproc::drawing::draw_filled_circle(
            &image,
            (x, y),
            radius as i32,
            Rgba([255, 255, 255, 255]),
        )
    }

    image
}

fn generate_village(scenery: &Scenery) -> VillageOperationResult<Village> {
    let mut rng = rand::thread_rng();

    let mut village = Village::default();

    for x in 0..scenery.params().plate_width_cells as i16 {
        for y in 0..scenery.params().plate_height_cells as i16 {
            let building_type: BuildingType = rng.gen();
            let level = rng.gen_range(building_type.level_range());
            let cell = Cell::new(x, y);

            let building = Building {
                building_type,
                level,
                characteristics: BuildingCharacteristics::Passive,
            };

            match village.add_component(cell, Component {
                kind: ComponentType::Building(building),
                life_points: None,
            }) {
                Ok(_) => {}
                Err(VillageOperationError::ComponentCollides) => {} //do not panic if the building collides with another one
            };
        }
    }

    Ok(village)
}

fn create_village() -> VillageOperationResult<Village> {
    let mut village = Village::default();

    village.add_component(Cell::new(10, 10), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(0,0), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(1,0), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(2,0), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(3,0), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(4,0), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(4,1), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(4,2), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(4,3), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(11, 10), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(11, 11), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(12, 10), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(10, 12), Component {
        kind: ComponentType::Wall(Wall {
            level: 10
        }),
        life_points: Some(20.0)
    })?;

    village.add_component(Cell::new(20, 15), Component {
        kind: ComponentType::Building(Building {
            building_type: BuildingType::ArmyCamp,
            level: 3,
            characteristics: BuildingCharacteristics::Passive,
        }),
        life_points: Some(400f32),
    })?;

    village.add_component(Cell::new(3, 6), Component {
        kind: ComponentType::Building(Building {
            building_type: BuildingType::Mortar(MissileDefenceState::Regular),
            level: 13,
            characteristics: BuildingCharacteristics::Passive,
        }),
        life_points: Some(250f32),
    })?;


    village.add_component(Cell::new(0, 1), Component {
        kind: ComponentType::Building(Building {
            building_type: BuildingType::BuilderHut,
            level: 1,
            characteristics: BuildingCharacteristics::Passive,
        }),
        life_points: Some(360f32),
    })?;

    village.add_component(Cell::new(8, 15), Component {
        kind: ComponentType::Building(Building {
            building_type: BuildingType::ArcherTower(ArcherDefenceState::Regular),
            level: 12,
            characteristics: BuildingCharacteristics::Passive,
        }),
        life_points: Some(1000f32),
    })?;

    village.add_component(Cell::new(7, 12), Component {
        kind: ComponentType::Building(Building {
            building_type: BuildingType::HiddenTesla,
            level: 9,
            characteristics: BuildingCharacteristics::Passive,
        }),
        life_points: Some(780f32),
    })?;

    village.add_component(Cell::new(3, 3), Component {
        kind: ComponentType::Building(Building {
            building_type: BuildingType::HiddenTesla,
            level: 9,
            characteristics: BuildingCharacteristics::Passive,
        }),
        life_points: Some(47f32),
    })?;

    village.add_component(Cell::new(20, 35), Component {
        kind: ComponentType::Building(Building {
            building_type: BuildingType::TownHall,
            level: 11,
            characteristics: BuildingCharacteristics::Passive,
        }),
        life_points: Some(186f32),
    })?;

    Ok(village)
}
