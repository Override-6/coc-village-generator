use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::collections::HashMap;

use crate::cell::Cell;
use crate::position::Pos;
use crate::scenery::Scenery;
use crate::troop::Troop;
use crate::village::{Component, ComponentId, is_defensive_building, Village};

#[derive(Debug, PartialEq, Eq)]
struct Node {
    position: Pos,
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

fn neighbors(point: Pos, width: i16, height: i16) -> Vec<Pos> {
    let mut result = vec![];

    let potential_neighbors = vec![
        Pos {
            x: point.x + 1.0,
            y: point.y,
        },
        Pos {
            x: point.x - 1.0,
            y: point.y,
        },
        Pos {
            x: point.x,
            y: point.y + 1.0,
        },
        Pos {
            x: point.x,
            y: point.y - 1.0,
        },
    ];

    for neighbor in potential_neighbors {
        if neighbor.x >= 0.0 && neighbor.x <= width as f32 && neighbor.y >= 0.0 && neighbor.y <= height as f32 {
            result.push(neighbor);
        }
    }

    result
}

fn reconstruct_path(came_from: &HashMap<Pos, Pos>, current: Pos) -> Vec<Pos> {
    let mut path = vec![current];
    let mut current = current;
    while let Some(&previous) = came_from.get(&current) {
        current = previous;
        path.push(current);
    }
    path.reverse();
    path
}

fn pathfind(start: Pos, goal: Pos, village: &Village, scenery: &Scenery) -> Vec<Pos> {
    let width = scenery.params().plate_width_cells as i16;
    let height = scenery.params().plate_height_cells as i16;

    let mut open_set = BinaryHeap::new();
    open_set.push(Node {
        position: start,
        cost: 0,
    });

    let mut came_from: HashMap<Pos, Pos> = HashMap::new();
    let mut g_score: HashMap<Pos, i32> = HashMap::new();
    let mut closed_set: HashSet<Pos> = HashSet::new();

    g_score.insert(start, 0);

    while let Some(Node {
        position: current, ..
    }) = open_set.pop()
    {
        if current == goal {
            return reconstruct_path(&came_from, current);
        }

        closed_set.insert(current);

        for neighbor in neighbors(current, width, height) {
            if goal != neighbor
                && (closed_set.contains(&neighbor) || village.is_cell_blocked(neighbor.to_cell()))
            {
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

fn list_cells_of_component(cell: Cell, comp: &Component) -> Vec<Cell> {
    let building_diameter = comp.get_plot_size().cell_diameter();

    let mut cells = Vec::new();

    for x in 0..building_diameter {
        for y in 0..building_diameter {
            cells.push(Cell {
                x: cell.x + x as i16,
                y: cell.y + y as i16,
            });
        }
    }

    cells
}

pub fn find_route_to_next_building(
    troop: &Troop,
    village: &Village,
    scenery: &Scenery,
) -> Option<(Vec<Pos>, ComponentId)> {
    #[derive(Copy, Clone)]
    struct ClosestBuildingCellInfo {
        id: ComponentId,
        pos: Pos,
        distance: f32,
    }

    let mut closest_building: Option<ClosestBuildingCellInfo> = None;

    // find the closest building cell
    for (comp_id, cell, comp) in village.iter_components() {
        if comp.life_points.is_some_and(|lp| lp == 0f32) {
            continue; //ignore dead buildings
        }
        
        if village.state().remaining_defenses > 0 && is_defensive_building(comp) {
            //if there is no defense left, then target other buildings
            continue
        }

        for building_cell in list_cells_of_component(cell, comp) {
            let distance = building_cell.to_pos().distance(troop.pos);
            if closest_building.is_none() || closest_building.is_some_and(|b| distance < b.distance)
            {
                closest_building = Some(ClosestBuildingCellInfo {
                    distance,
                    id: comp_id,
                    pos: building_cell.to_pos(),
                });
            }
        }
    }

    closest_building.map(|closest_building| {
        (
            pathfind(troop.pos, closest_building.pos, village, scenery),
            closest_building.id,
        )
    })
}
