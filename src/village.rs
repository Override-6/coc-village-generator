use std::fmt::Debug;

use crate::buidling::{Building, BuildingCharacteristics, PlotSize};
use crate::cell::Cell;
use crate::scenery::Scenery;
use crate::wall::{Wall, WallConnectionType};

#[derive(Clone)]
pub struct Village {
    grid: Vec<Option<Box<Component>>>,
    state: State,
    scenery: Scenery,
}

#[derive(Clone)]
pub enum ComponentType {
    Building(Building),
    Wall(Wall),
}

#[derive(Clone)]
pub struct Component {
    pub life_points: Option<f32>,
    pub kind: ComponentType,
}

impl Component {
    pub fn is_destroyed(&self) -> bool {
        self.life_points.is_some_and(|lp| lp == 0f32)
    }

    pub fn get_plot_size(&self) -> PlotSize {
        match &self.kind {
            ComponentType::Building(b) => b.building_type.plot_size(),
            ComponentType::Wall(_) => PlotSize::X1Invisible
        }
    }
}

#[derive(Default, Clone)]
pub struct State {
    pub remaining_defenses: usize,
}

pub type VillageOperationResult<R> = Result<R, VillageOperationError>;

#[derive(Debug)]
pub enum VillageOperationError {
    ComponentCollides,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ComponentId(usize);

impl Default for Village {
    fn default() -> Self {
        let scenery = Scenery::default();
        let grid_size = (scenery.params().plate_width_cells as usize + 1) * (scenery.params().plate_height_cells as usize + 1);
        Self {
            grid: vec![None; grid_size],
            state: Default::default(),
            scenery,
        }
    }
}

impl Village {
    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn scenery(&self) -> &Scenery {
        &self.scenery
    }

    fn can_plot_fit(&self, at: Cell, plot_size: PlotSize) -> bool {
        let plot_diameter = plot_size.cell_diameter();

        for x in 0..plot_diameter {
            for y in 0..plot_diameter {
                let cell = Cell::new(at.x + x as i16, at.y + y as i16);
                if self.get_component_at(cell).is_some() {
                    return false;
                }
            }
        }

        true
    }

    pub fn add_component(&mut self, cell: Cell, component: Component) -> VillageOperationResult<ComponentId> {
        if !self.can_plot_fit(cell, component.get_plot_size()) {
            return Err(VillageOperationError::ComponentCollides);
        }

        if is_defensive_building(&component) {
            self.state.remaining_defenses += 1;
        }

        let idx = self.get_cell_idx(cell);
        self.grid[idx] = Some(Box::new(component));

        Ok(ComponentId(idx))
    }

    pub fn get_wall_connection_type(&self, cell: Cell) -> Option<WallConnectionType> {
        
        if !self.get_component_at(cell).is_some_and(|(_, c)| is_wall(c)) {
            return None
        }
        
        let comp_left = cell.x > 0 && self.get_component_at(Cell::new(cell.x - 1, cell.y))
            .is_some_and(|(_, c)| is_wall(c));
        let comp_right = cell.y < self.scenery.params().plate_height_cells as i16 && self.get_component_at(Cell::new(cell.x, cell.y + 1))
            .is_some_and(|(_, c)| is_wall(c));

        let kind = if comp_left && comp_right {
            WallConnectionType::RightLeft
        } else if comp_left {
            WallConnectionType::LeftOnly
        } else if comp_right {
            WallConnectionType::RightOnly
        } else {
            WallConnectionType::Lonely
        };
        
        Some(kind)
    }

    /// returns true if the building got destroyed
    pub fn damage_component(&mut self, damages: f32, comp_id: ComponentId) -> bool {
        let Some(component) = &mut self.get_component_mut(comp_id) else {
            panic!("component not found")
        };

        match component.life_points {
            Some(lp) => {
                let lp = (lp - damages).max(0f32);
                component.life_points = Some(lp);
                let building_destroyed = lp == 0f32;
                if building_destroyed && is_defensive_building(&component) {
                    self.state.remaining_defenses -= 1;
                }

                building_destroyed
            }
            None => panic!("cannot apply damages on a building with undefined lifepoints !"),
        }
    }

    fn get_cell_idx(&self, cell: Cell) -> usize {
        cell.x as usize * self.scenery.params().plate_width_cells as usize + cell.y as usize
    }

    fn get_component_at(&self, cell: Cell) -> Option<(ComponentId, &Component)> {
        let idx = self.get_cell_idx(cell);
        self.get_component(ComponentId(idx)).map(|v| (ComponentId(idx), v))
    }

    fn get_component(&self, id: ComponentId) -> Option<&Component> {
        self.grid[id.0].as_ref().map(|b| b.as_ref())
    }

    fn get_component_mut(&mut self, id: ComponentId) -> Option<&mut Component> {
        self.grid[id.0].as_mut().map(|b| b.as_mut())
    }

    pub fn is_component_destroyed(&self, comp_id: ComponentId) -> bool {
        self.get_component_lifepoints(comp_id)
            .is_some_and(|lp| lp == 0.0)
    }

    pub fn get_component_lifepoints(&self, comp_id: ComponentId) -> Option<f32> {
        self.get_component(comp_id).and_then(|c| c.life_points)
    }

    pub fn is_cell_blocked(&self, cell: Cell) -> bool {
        self.get_component_at(cell).is_some_and(|(_, c)| c.is_destroyed())
    }

    pub fn iter_components(&self) -> impl Iterator<Item=(ComponentId, Cell, &Component)> {
        self.grid
            .iter()
            .enumerate()
            .filter_map(|(idx, b)| b.as_ref().map(|bx| (idx, bx.as_ref())))
            .map(|(idx, b)| {
                (ComponentId(idx), Cell {
                    x: (idx / self.scenery.params().plate_width_cells as usize) as i16,
                    y: (idx % self.scenery.params().plate_width_cells as usize) as i16,
                }, b)
            })
    }
}

pub fn is_defensive_building(comp: &Component) -> bool {
    matches!(
        comp,
        Component {
        kind: ComponentType::Building(Building {
            characteristics: BuildingCharacteristics::Defense(_),
            ..
        }),
        ..
    })
}

fn is_wall(comp: &Component) -> bool {
    matches!(
        comp,
        Component {
        kind: ComponentType::Wall(_),
        ..
    })
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
