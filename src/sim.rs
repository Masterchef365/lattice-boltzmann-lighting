use ndarray::Array2;

pub struct Sim {
    light: Array2<Cell>,
    env: Array2<Environment>
}

pub enum Environment {
    Wall,
    Fog(f32),
}

pub struct Cell {
    dirs: [f32; 9],
}
