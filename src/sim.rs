use egui_pixel_editor::image::PixelInterface;
use ndarray::Array2;

pub struct Sim {
    pub light: Array2<Cell>,
    pub env: Array2<Environment>
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Environment {
    Wall,
    Fog(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(Default)]
pub struct Cell {
    pub dirs: [f32; 9],
}

impl Sim {
    pub fn new(width: usize, height: usize) -> Self {
        let mut light = Array2::from_elem((width, height), Cell::default());
        let mut env = Array2::from_elem((width, height), Environment::Fog(1.0));
        env.slice_mut(ndarray::s![.., height - 1]).fill(Environment::Wall);
        env.slice_mut(ndarray::s![width - 1, ..]).fill(Environment::Wall);
        env.slice_mut(ndarray::s![.., 0]).fill(Environment::Wall);
        env.slice_mut(ndarray::s![0, ..]).fill(Environment::Wall);

        light.slice_mut(ndarray::s![50..=70, 50..=70]).fill(Cell { dirs: [1.0; 9] });

        Self {
            light,
            env,
        }
    }
}

impl PixelInterface for Environment {
    fn as_rgba(&self) -> egui::Color32 {
        match self {
            Self::Wall => egui::Color32::RED,
            Self::Fog(_) => egui::Color32::TRANSPARENT,
        }
    }
}

impl PixelInterface for Cell {
    fn as_rgba(&self) -> egui::Color32 {
        egui::Color32::from_gray((self.dirs.iter().sum::<f32>() * 255.0).clamp(0.0, 255.0) as u8)
    }
}
