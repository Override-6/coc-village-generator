use enum_assoc::Assoc;

#[derive(Clone)]
pub struct Wall {
    pub level: u8,
}

#[derive(Assoc)]
#[func(pub fn name(& self) -> & str)]
#[func(pub fn size_ratio(& self) -> f32)]
#[func(pub fn width_shift_ratio(& self) -> f32)]
#[func(pub fn height_shift_ratio(& self) -> f32)]
pub enum WallConnectionType {
    #[assoc(name = "lonely")]
    #[assoc(size_ratio = 0.5)]
    #[assoc(width_shift_ratio = 0.25)]
    #[assoc(height_shift_ratio = 1.0)]
    Lonely,

    #[assoc(name = "corner")]
    #[assoc(size_ratio = 0.5)]
    #[assoc(width_shift_ratio = -0.25)]
    #[assoc(height_shift_ratio = 0.7)]
    RightLeft,

    #[assoc(name = "connected_left")]
    #[assoc(size_ratio = 0.5)]
    #[assoc(width_shift_ratio = -0.15)]
    #[assoc(height_shift_ratio = 0.8)]
    LeftOnly,

    #[assoc(name = "connected_right")]
    #[assoc(size_ratio = 0.5)]
    #[assoc(width_shift_ratio = -0.15)]
    #[assoc(height_shift_ratio = 0.8)]
    RightOnly,
}
