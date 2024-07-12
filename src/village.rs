use std::fmt::Debug;

use crate::buidling::{Building, BuildingCharacteristics, PlotSize};
use crate::cell::Cell;

#[derive(Default, Clone)]
pub struct Village {
    buildings: Vec<Building>,
    state: State,
}

#[derive(Default, Clone)]
pub struct State {
    pub remaining_defenses: usize,
}

pub type VillageOperationResult<R> = Result<R, VillageOperationError>;

#[derive(Debug)]
pub enum VillageOperationError {
    BuildingCollides
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct BuildingId(pub usize);

impl Village {
    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn add_building(&mut self, building: Building) -> VillageOperationResult<BuildingId> {
        for village_building in &self.buildings {
            if collides(village_building.pos, village_building.building_type.plot_size(), building.pos, building.building_type.plot_size()) {
                return Err(VillageOperationError::BuildingCollides);
            }
        }

        if let BuildingCharacteristics::Defense(_) = building.characteristics {
            self.state.remaining_defenses += 1;
        }

        let id = BuildingId(self.buildings.len());

        self.buildings.push(building);

        Ok(id)
    }

    /// returns true if the building got destroyed
    pub fn damage_building(&mut self, damages: f32, building_id: BuildingId) -> bool {
        let building = &mut self.buildings[building_id.0];

        match building.life_points {
            Some(lp) => {
                let lp = (lp - damages).max(0f32);
                building.life_points = Some(lp);
                let building_destroyed = lp == 0f32;
                if building_destroyed && matches!(building.characteristics, BuildingCharacteristics::Defense(_)) {
                    self.state.remaining_defenses -= 1;
                }

                building_destroyed
            }
            None => panic!("cannot apply damages on a building with undefined lifepoints !")
        }
    }
    
    pub fn is_building_destroyed(&self, building_id: BuildingId) -> bool {
        self.get_building_lifepoints(building_id).is_some_and(|lp| lp == 0.0)
    }

    pub fn get_building_lifepoints(&self, building_id: BuildingId) -> Option<f32> {
        self.buildings[building_id.0].life_points
    }
    
    pub fn is_cell_blocked(&self, cell: Cell) -> bool {
        for village_building in &self.buildings {
            if village_building.is_destroyed() {
                continue;
            }

            if collides(village_building.pos, village_building.building_type.plot_size(), cell, PlotSize::X1Invisible) {
                return true;
            }
        }

        false
    }

    pub fn iter_buildings(&self) -> impl Iterator<Item=(BuildingId, &Building)> {
        self.buildings.iter()
            .enumerate()
            .map(|(idx, b)| (BuildingId(idx), b))
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

