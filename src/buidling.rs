use std::ops::RangeInclusive;
use enum_assoc::Assoc;
use rand_derive2::RandGen;
use crate::cell::Cell;


#[derive(Assoc, Clone, RandGen)]
#[func(pub fn name(& self) -> & 'static str)]
#[func(pub fn plot_size(& self) -> PlotSize)]
#[func(pub fn self_size(& self) -> PlotSize { self.plot_size() })]
#[func(pub fn level_range(& self) -> RangeInclusive < u8 >)]
pub enum BuildingType {
    #[assoc(name = "Air_Bomb")]
    #[assoc(plot_size = PlotSize::X1Invisible)]
    #[assoc(level_range = 9..=9)]
    AirBomb,

    #[assoc(name = "Archer_Tower")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(self_size = PlotSize::X2)]
    #[assoc(level_range = 1..=21)]
    ArcherTower(ArcherDefenceState),

    #[assoc(name = "Army_Camp")]
    #[assoc(plot_size = PlotSize::X4)]
    #[assoc(self_size = PlotSize::X2)]
    #[assoc(level_range = 3..=12)]
    ArmyCamp,

    #[assoc(name = "Barracks")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 3..=17)]
    Barracks,

    #[assoc(name = "Blacksmith")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 5..=5)]
    Blacksmith,

    #[assoc(name = "Builders_Hut")]
    #[assoc(plot_size = PlotSize::X2)]
    #[assoc(level_range = 2..=6)]
    BuilderHut,

    #[assoc(name = "Cannon")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(self_size = PlotSize::X3)]
    #[assoc(level_range = 1..=21)]
    Cannon(MissileDefenceState),

    #[assoc(name = "Dark_Barracks")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 3..=11)]
    DarkBarracks,

    #[assoc(name = "Dark_Elixir_Drill")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 1..=10)]
    DarkElixirDrill,

    #[assoc(name = "Dark_Elixir_Storage")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 3..=11)]
    DarkElixirStorage,

    #[assoc(name = "Dark_Spell_Factory")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 2..=6)]
    DarkSpellFactory(SpellFactoryState),

    #[assoc(name = "Eagle_Artillery")]
    #[assoc(plot_size = PlotSize::X4)]
    #[assoc(self_size = PlotSize::X3)]
    #[assoc(level_range = 2..=7)]
    EagleArtillery(EagleArtilleryState),

    #[assoc(name = "Elixir_Collector")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(self_size = PlotSize::X2)]
    #[assoc(level_range = 1..=16)]
    ElixirCollector(ContainerState),

    #[assoc(name = "Elixir_Storage")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 3..=17)]
    ElixirStorage(ContainerState),

    #[assoc(name = "Giant_Bomb")]
    #[assoc(plot_size = PlotSize::X2Invisible)]
    #[assoc(level_range = 5..=5)]
    GiantBomb(ExplosiveState),

    #[assoc(name = "Gold_Mine")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 1..=16)]
    GoldMine,

    #[assoc(name = "Gold_Storage")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 3..=17)]
    GoldStorage(ContainerState),

    #[assoc(name = "Hidden_Tesla")]
    #[assoc(plot_size = PlotSize::X1Invisible)]
    #[assoc(level_range = 1..=15)]
    HiddenTesla,

    #[assoc(name = "Inferno_Tower")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(self_size = PlotSize::X2)]
    #[assoc(level_range = 1..=10)]
    InfernoTower(InfernoState),

    #[assoc(name = "Laboratory")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 3..=14)]
    Laboratory,

    #[assoc(name = "Mortar")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 1..=16)]
    Mortar(MissileDefenceState),

    #[assoc(name = "Pet_House")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 3..=10)]
    PetHouse,

    #[assoc(name = "Scattershot")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 3..=5)]
    Scattershot(ScattershotState),

    #[assoc(name = "Seeking_Air_Mine")]
    #[assoc(plot_size = PlotSize::X1Invisible)]
    #[assoc(self_size = PlotSize::X1Invisible)]
    #[assoc(level_range = 5..=5)]
    SeekingAirMine,

    #[assoc(name = "Spell_Factory")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 1..=7)]
    SpellFactory(SpellFactoryState),

    #[assoc(name = "Town_Hall")]
    #[assoc(plot_size = PlotSize::X4)]
    #[assoc(self_size = PlotSize::X3)]
    #[assoc(level_range = 3..=16)]
    TownHall,

    #[assoc(name = "Wizard_Tower")]
    #[assoc(plot_size = PlotSize::X3)]
    #[assoc(level_range = 3..=16)]
    WizardTower,

    #[assoc(name = "Workshop")]
    #[assoc(plot_size = PlotSize::X4)]
    #[assoc(level_range = 3..=7)]
    Workshop,

}

pub const BUILDING_ASSETS_FOLDER: &str = "datasets/buildings/images/train";

impl BuildingType {
    pub fn get_file_path(&self, level: u8) -> String {
        let file_name = self.get_file_name(level);

        format!("{BUILDING_ASSETS_FOLDER}/{file_name}")
    }

    pub fn get_file_name(&self, level: u8) -> String {
        match self {
            BuildingType::ArcherTower(state) => self.make_file_name(Some(level), Some(*state)),
            BuildingType::Cannon(state) => self.make_file_name(Some(level), Some(*state)),
            BuildingType::DarkSpellFactory(state) => self.make_file_name(Some(level), Some(*state)),
            BuildingType::EagleArtillery(state) => self.make_file_name(Some(level), Some(*state)),
            BuildingType::ElixirCollector(state) => self.make_file_name(Some(level), Some(*state)),
            BuildingType::ElixirStorage(state) => self.make_file_name(Some(level), Some(*state)),
            BuildingType::GiantBomb(state) => self.make_file_name(Some(level), Some(*state)),
            BuildingType::GoldStorage(state) => self.make_file_name(Some(level), Some(*state)),
            BuildingType::InfernoTower(state) => self.make_file_name(Some(level), Some(*state)),
            BuildingType::Mortar(state) => self.make_file_name(Some(level), Some(*state)),
            BuildingType::Scattershot(state) => self.make_file_name(Some(level), Some(*state)),
            BuildingType::SpellFactory(state) => self.make_file_name(Some(level), Some(*state)),
            BuildingType::BuilderHut if level == 1 => self.make_file_name(None, None::<ArcherDefenceState>),
            BuildingType::TownHall if level >= 12 && level < 16 => {
                self.make_file_name(Some(level), Some("-1"))
            }
            _ => self.make_file_name(Some(level), None::<ArcherDefenceState>)
        }
    }

    fn make_file_name(&self, level: Option<u8>, state: Option<impl DefenceState>) -> String {
        let level_str = level.map(|lvl| lvl.to_string()).unwrap_or_default();
        format!("{}{level_str}{}.png", self.name(), state.map(DefenceState::get_as_suffix).unwrap_or(""))
    }
}

impl DefenceState for &'static str {
    fn get_as_suffix(self) -> &'static str {
        self
    }
}

#[derive(Assoc, Copy, Clone, Eq, PartialEq)]
#[func(pub const fn plot_file(& self) -> Option < & 'static str >)]
#[func(pub const fn cell_diameter(& self) -> u8)]
pub enum PlotSize {
    #[assoc(cell_diameter = 1)]
    X1Invisible,

    #[assoc(plot_file = "assets/plots/2x2.png")]
    #[assoc(cell_diameter = 2)]
    X2,

    #[assoc(cell_diameter = 2)]
    X2Invisible,

    #[assoc(plot_file = "assets/plots/3x3.png")]
    #[assoc(cell_diameter = 3)]
    X3,

    #[assoc(plot_file = "assets/plots/4x4.png")]
    #[assoc(cell_diameter = 4)]
    X4,
}

trait DefenceState {
    fn get_as_suffix(self) -> &'static str;
}

#[derive(Default, Copy, Clone, RandGen)]
pub enum ArcherDefenceState {
    #[default]
    Regular,
    // LongRange,
    // ShortRange,
}


impl DefenceState for ArcherDefenceState {
    fn get_as_suffix(self) -> &'static str {
        match self {
            ArcherDefenceState::Regular => "",
            // ArcherDefenceState::LongRange => "UP",
            // ArcherDefenceState::ShortRange => "G"
        }
    }
}

#[derive(Default, Copy, Clone, RandGen)]
pub enum MissileDefenceState {
    #[default]
    Regular,
    // LongRange,
    // ShortRange,
}

impl DefenceState for MissileDefenceState {
    fn get_as_suffix(self) -> &'static str {
        match self {
            MissileDefenceState::Regular => "",
            // MissileDefenceState::LongRange => "B",
            // MissileDefenceState::ShortRange => "G"
        }
    }
}

#[derive(Default, Copy, Clone, RandGen)]
pub enum EagleArtilleryState {
    #[default]
    Loaded,
    HeadDown,
    Unloaded,
}

impl DefenceState for EagleArtilleryState {
    fn get_as_suffix(self) -> &'static str {
        match self {
            EagleArtilleryState::Loaded => "",
            EagleArtilleryState::HeadDown => "_Head_Down",
            EagleArtilleryState::Unloaded => "_Unloaded"
        }
    }
}


#[derive(Default, Copy, Clone, RandGen)]
pub enum SpellFactoryState {
    #[default]
    Inactive,
    Active,
}

impl DefenceState for SpellFactoryState {
    fn get_as_suffix(self) -> &'static str {
        match self {
            SpellFactoryState::Inactive => "",
            SpellFactoryState::Active => "_Active"
        }
    }
}


#[derive(Default, Copy, Clone, RandGen)]
pub enum ContainerState {
    #[default]
    Empty,
    // Full,
}

impl DefenceState for ContainerState {
    fn get_as_suffix(self) -> &'static str {
        match self {
            ContainerState::Empty => "",
            // ContainerState::Full => "B"
        }
    }
}

#[derive(Default, Copy, Clone, RandGen)]
pub enum ExplosiveState {
    #[default]
    Armed,
    Unarmed,
}

impl DefenceState for ExplosiveState {
    fn get_as_suffix(self) -> &'static str {
        match self {
            ExplosiveState::Armed => "",
            ExplosiveState::Unarmed => "_unarmed"
        }
    }
}

#[derive(Copy, Clone, RandGen)]
pub enum InfernoState {
    Multi,
    MultiDepleted,
    Single,
    SingleDepleted,
}

impl DefenceState for InfernoState {
    fn get_as_suffix(self) -> &'static str {
        match self {
            InfernoState::Multi => "_Multi",
            InfernoState::MultiDepleted => "_Multi_Depleted",
            InfernoState::Single => "_Single",
            InfernoState::SingleDepleted => "_Single_Depleted",
        }
    }
}

#[derive(Copy, Default, Clone, RandGen)]
pub enum ScattershotState {
    #[default]
    Regular,
    Depleted,
}

impl DefenceState for ScattershotState {
    fn get_as_suffix(self) -> &'static str {
        match self {
            ScattershotState::Regular => "",
            ScattershotState::Depleted => "_Depleted",
        }
    }
}

pub struct Building {
    pub building_type: BuildingType,
    pub pos: Cell,
    pub level: u8,
}

