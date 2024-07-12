use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::collections::HashMap;

use crate::buidling::{Building, BuildingCharacteristics};
use crate::cell::Cell;
use crate::scenery::Scenery;
use crate::troop::Troop;
use crate::village::{BuildingId, Village};

#[derive(Debug, PartialEq, Eq)]
struct Node {
    position: Cell,
    cost: i32,
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

fn neighbors(point: Cell, width: i16, height: i16) -> Vec<Cell> {
    let mut result = vec![];

    let potential_neighbors = vec![
        Cell { x: point.x + 1, y: point.y },
        Cell { x: point.x - 1, y: point.y },
        Cell { x: point.x, y: point.y + 1 },
        Cell { x: point.x, y: point.y - 1 },
    ];

    for neighbor in potential_neighbors {
        if neighbor.x >= 0 && neighbor.x <= width && neighbor.y >= 0 && neighbor.y <= height {
            result.push(neighbor);
        }
    }

    result
}

fn reconstruct_path(came_from: &HashMap<Cell, Cell>, current: Cell) -> Vec<Cell> {
    let mut path = vec![current];
    let mut current = current;
    while let Some(&previous) = came_from.get(&current) {
        current = previous;
        path.push(current);
    }
    path.reverse();
    path
}

fn pathfind(start: Cell, goal: Cell, village: &Village, scenery: &Scenery) -> Vec<Cell> {
    let width = scenery.params().plate_width_cells as i16;
    let height = scenery.params().plate_height_cells as i16;

    let mut open_set = BinaryHeap::new();
    open_set.push(Node {
        position: start,
        cost: 0,
    });

    let mut came_from: HashMap<Cell, Cell> = HashMap::new();
    let mut g_score: HashMap<Cell, i32> = HashMap::new();
    let mut closed_set: HashSet<Cell> = HashSet::new();

    g_score.insert(start, 0);

    while let Some(Node { position: current, .. }) = open_set.pop() {
        if current == goal {
            return reconstruct_path(&came_from, current);
        }

        closed_set.insert(current);

        for neighbor in neighbors(current, width, height) {
            if goal != neighbor && (closed_set.contains(&neighbor) || village.is_cell_blocked(neighbor)) {
                continue;
            }

            let tentative_g_score = g_score[&current] + 1;
            if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g_score);
                open_set.push(Node {
                    position: neighbor,
                    cost: tentative_g_score,
                });
            }
        }
    }

    vec![]
}


fn list_cells_of_building(building: &Building) -> Vec<Cell> {
    let pos = building.pos;
    let building_diameter = building.building_type.plot_size().cell_diameter();

    let mut cells = Vec::new();

    for x in 0..building_diameter {
        for y in 0..building_diameter {
            cells.push(Cell { x: pos.x + x as i16, y: pos.y + y as i16 });
        }
    }

    cells
}

pub fn find_route_to_next_building(troop: &Troop, village: &Village, scenery: &Scenery) -> Option<(Vec<Cell>, BuildingId)> {
    #[derive(Copy, Clone)]
    struct ClosestBuildingCellInfo {
        id: BuildingId,
        cell: Cell,
        distance: f32,
    }

    let mut closest_building: Option<ClosestBuildingCellInfo> = None;

    // find the closest building cell
    for (building_id, building) in village.iter_buildings() {
        if building.life_points.is_some_and(|lp| lp == 0f32) {
            continue; //ignore dead buildings
        }

        match building.characteristics {
            BuildingCharacteristics::Passive => {
                //if there is no defense left, then target other buildings
                if village.state().remaining_defenses > 0 && troop.tpe.prefer_defenses() {
                    continue;
                }
            }
            BuildingCharacteristics::Defense(_) => {}
        }

        for building_cell in list_cells_of_building(building) {
            let distance = building_cell.distance(troop.pos);
            if closest_building.is_none() || closest_building.is_some_and(|b| distance < b.distance) {
                closest_building = Some(ClosestBuildingCellInfo {
                    distance,
                    id: building_id,
                    cell: building_cell,
                });
            }
        }
    }

    closest_building.map(|closest_building| {
        (pathfind(troop.pos, closest_building.cell, village, scenery), closest_building.id)
    })
}
