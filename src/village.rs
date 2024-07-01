use std::fmt::Debug;

use crate::buidling::{Building, PlotSize};
use crate::cell::Cell;

#[derive(Default)]
pub struct Village {
    buildings: Vec<Building>,
}

pub type VillageOperationResult<R> = Result<R, VillageOperationError>;

#[derive(Debug)]
pub enum VillageOperationError {
    BuildingCollides
}

impl Village {
    pub fn add_building(&mut self, building: Building) -> VillageOperationResult<()> {
        for village_building in &self.buildings {
            if collides(village_building.pos, village_building.building_type.plot_size(), building.pos, building.building_type.plot_size()) {
                return Err(VillageOperationError::BuildingCollides);
            }
        }

        self.buildings.push(building);

        Ok(())
    }

    pub fn iter_buildings(&self) -> impl Iterator<Item=&Building> {
        self.buildings.iter()
    }
}

const fn collides(pos_a: Cell, size_a: PlotSize, pos_b: Cell, size_b: PlotSize) -> bool {
    if pos_b.x >= pos_a.x + size_a.cell_diameter() as i16 {
        return false;
    }

    if pos_a.x >= pos_b.x + size_b.cell_diameter() as i16 {
        return false;
    }

    if pos_b.y >= pos_a.y + size_a.cell_diameter() as i16 {
        return false;
    }

    if pos_a.y >= pos_b.y + size_b.cell_diameter() as i16 {
        return false;
    }

    true
}

