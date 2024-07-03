use imageproc::point::Point;

pub struct SceneryParams {
    pub image_path: String,
    pub bottom_left_corner: Point<i32>,
    pub upper_left_corner: Point<i32>,
    pub upper_right_corner: Point<i32>,
    pub bottom_right_corner: Point<i32>,

    pub plate_width_cells: u8,
    pub plate_height_cells: u8,
    pub base_cell_position: Point<i32>,
}

pub struct Scenery {
    params: SceneryParams,

    // calculated data from parameters fields
    cell_width: f32,
    cell_height: f32,
}

impl From<SceneryParams> for Scenery {
    fn from(value: SceneryParams) -> Self {
        Self {
            cell_width: (value.bottom_right_corner.x - value.bottom_left_corner.x) as f32 / value.plate_width_cells as f32,
            cell_height: (value.upper_right_corner.y - value.upper_left_corner.y) as f32 / value.plate_height_cells as f32,
            params: value,
        }
    }
}

impl Scenery {
    pub fn params(&self) -> &SceneryParams {
        &self.params
    }
    pub fn cell_width(&self) -> f32 {
        self.cell_width
    }
    pub fn cell_height(&self) -> f32 {
        self.cell_height
    }
    
    pub fn get_plate_x_axis(&self, x: f32, y: f32) -> f32 {
        let x_change = (self.params.bottom_left_corner.x - self.params.bottom_right_corner.x) as f32;
        let y_change = (self.params.bottom_left_corner.y - self.params.bottom_right_corner.y) as f32;

        let gradient = x_change / y_change;


        self.params.base_cell_position.x as f32 + (gradient * x) + y
    }

    pub fn get_plate_y_axis(&self, x: f32, y: f32) -> f32 {
        let x_change = (self.params.bottom_left_corner.x - self.params.upper_left_corner.x) as f32;
        let y_change = (self.params.bottom_left_corner.y - self.params.upper_left_corner.y) as f32;

        let gradient = y_change / x_change;

        self.params.base_cell_position.y as f32 + (gradient * y) + x
    }
}

pub fn default_scenery() -> Scenery {
    let bottom_left_corner = Point { x: 229, y: 910 };
    let upper_left_corner = Point { x: 1215, y: 173 };
    let upper_right_corner = Point { x: 2200, y: 910 };
    let bottom_right_corner = Point { x: 1215, y: 1650 };

    Scenery::from(SceneryParams {
        image_path: "assets/scenery.png".to_string(),

        bottom_left_corner,
        upper_left_corner,
        upper_right_corner,
        bottom_right_corner,
        plate_width_cells: 44,
        plate_height_cells: 44,
        base_cell_position: bottom_left_corner,
    })
}

pub fn grid_scenery() -> Scenery {
    let bottom_left_corner = Point { x: 0, y: 640 };
    let upper_left_corner = Point { x: 0, y: 0 };
    let upper_right_corner = Point { x: 640, y: 0 };
    let bottom_right_corner = Point { x: 640, y: 640 };

    Scenery::from(SceneryParams {
        image_path: "assets/grid.png".to_string(),

        bottom_left_corner,
        upper_left_corner,
        upper_right_corner,
        bottom_right_corner,
        plate_width_cells: 44,
        plate_height_cells: 44,
        base_cell_position: bottom_left_corner,
    })
}