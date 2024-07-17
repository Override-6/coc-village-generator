#[derive(Default)]
pub struct Bounds {
    pub(crate) x_center: f32,
    pub(crate) y_center: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

pub struct Label {
    pub class: usize,
    pub bounds: Bounds,
}
