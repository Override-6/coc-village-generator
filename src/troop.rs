use enum_assoc::Assoc;
use image::Rgb;

use crate::cell::Cell;

#[derive(Clone)]
pub struct Troop {
    pub tpe: TroopType,
    pub pos: Cell,
}

#[derive(Assoc, Clone)]
#[func(pub fn prefer_defenses(& self) -> bool { true })]
#[func(pub fn range(& self) -> f32)]
#[func(pub fn damage_per_seconds(& self) -> f32)]
#[func(pub fn walk_speed(& self) -> f32)]
#[func(pub fn color(& self) -> Rgb < u8 >)]
pub enum TroopType {
    #[assoc(range = 0.0)]
    #[assoc(damage_per_seconds = 10.0)]
    #[assoc(walk_speed = 1.0)]
    #[assoc(color = Rgb([255, 0, 0]))]
    Barbarian,

    #[assoc(range = 4.0)]
    #[assoc(damage_per_seconds = 15.0)]
    #[assoc(walk_speed = 1.5)]
    #[assoc(color = Rgb([0, 255, 0]))]
    Archer,

    #[assoc(range = 0.0)]
    #[assoc(prefer_defenses = true)]
    #[assoc(damage_per_seconds = 30.0)]
    #[assoc(walk_speed = 0.5)]
    #[assoc(color = Rgb([0, 0, 255]))]
    Giant,
}

