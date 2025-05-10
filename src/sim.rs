use egui::Color32;
use egui_pixel_editor::image::PixelInterface;
use ndarray::Array2;

pub struct Sim {
    pub light: Array2<Cell>,
    pub light_source: Array2<Cell>,
    pub env: Array2<Environment>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Environment {
    pub scattering: f32,
    pub absorbtion: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Cell {
    pub dirs: [f32; 9],
}

/// Lattice-Boltzmann Lighting
/// Robert Geist, Karl Rasche, James Westall and Robert Schalkoff
///
/// Implemented here by Y.T.
impl Sim {
    pub fn new(width: usize, height: usize) -> Self {
        let light_source = Array2::from_elem((width, height), Cell::default());
        let mut light = Array2::from_elem((width, height), Cell::default());
        let air = Environment {
            scattering: 0.001,
            absorbtion: 0.0,
        };
        let mut env = Array2::from_elem((width, height), air);
        let wall = Environment {
            scattering: 1.0,
            absorbtion: 0.0,
        };
        env.slice_mut(ndarray::s![.., height - 1])
            .fill(wall);
        env.slice_mut(ndarray::s![width - 1, ..])
            .fill(wall);
        env.slice_mut(ndarray::s![.., 0]).fill(wall);
        env.slice_mut(ndarray::s![0, ..]).fill(wall);

        light
            .slice_mut(ndarray::s![50..=70, 50..=70])
            .fill(Cell { dirs: [1.0; 9] });

        Self {
            light,
            env,
            light_source,
        }
    }

    pub fn step(&mut self) {
        // Add light sources
        self.light
            .iter_mut()
            .zip(&self.light_source)
            .for_each(|(l, src)| {
                l.dirs
                    .iter_mut()
                    .zip(src.dirs)
                    .for_each(|(l, src)| *l += src);
            });

        // Distribute density locally
        // according to the collision rules
        for (src, env) in self.light.iter_mut().zip(&self.env) {
            let mut new_dense = [0_f32; 9];
            for in_idx in 0..9 {
                for out_idx in 0..9 {
                    new_dense[out_idx] += src.dirs[in_idx] * Θ(in_idx, out_idx, env);
                }
            }

            src.dirs = new_dense;
        }

        let mut dst = Array2::from_elem(self.light.dim(), Cell::default());

        // Now flow density to neighbors
        for (coord, src) in self.light.indexed_iter() {
            for in_idx in 0..9 {
                // Compute the index of the
                // node at i in the in direction
                if let Some(neigh) = compute_neighbor(coord, in_idx, &self.light) {
                    dst[neigh].dirs[in_idx] = src.dirs[in_idx];
                }
            }
        }

        self.light = dst;
    }
}

fn compute_neighbor(
    (x, y): (usize, usize),
    in_idx: usize,
    arr: &Array2<Cell>,
) -> Option<(usize, usize)> {
    const OFFSETS: [(isize, isize); 9] = [
        (-1, -1),
        (0, -1),
        (1, -1),

        (-1, 0),
        (0, 0),
        (1, 0),

        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let (width, height) = arr.dim();
    let (dx, dy) = OFFSETS[in_idx];

    // Bounds check
    if dx < 0 && x == 0 {
        return None;
    }
    if dx > 0 && x == width - 1 {
        return None;
    }
    if dy < 0 && y == 0 {
        return None;
    }
    if dy > 0 && y == height - 1 {
        return None;
    }

    Some(((x as isize + dx) as usize, (y as isize + dy) as usize))
}

impl PixelInterface for Environment {
    fn as_rgba(&self) -> egui::Color32 {
            let scattering_only = Color32::TRANSPARENT.lerp_to_gamma(Color32::CYAN, self.scattering);
            let scattering_and_absorbtion = Color32::RED.lerp_to_gamma(Color32::CYAN, self.scattering);
            scattering_only.lerp_to_gamma(scattering_and_absorbtion, self.absorbtion)
    }
}

impl PixelInterface for Cell {
    fn as_rgba(&self) -> egui::Color32 {
        egui::Color32::from_gray((self.dirs.iter().sum::<f32>() * 255.0).clamp(0.0, 255.0) as u8).additive()
    }
}

fn Θ(in_idx: usize, out_idx: usize, env: &Environment) -> f32 {
    let extinction_coeff = env.absorbtion + env.scattering;

    const CENTER_IDX: usize = 4;
    const IS_AXIAL: [bool; 9] = [
        true, false, true, //.
        false, false, false, //.
        true, false, true, //.
    ];

    if in_idx == CENTER_IDX {
        return if out_idx == CENTER_IDX {
            0.0
        } else {
            env.absorbtion
        };
    }

    if IS_AXIAL[in_idx] {
        if out_idx == CENTER_IDX {
            1.0 / 8.0
        } else if out_idx != in_idx {
            env.scattering / 8.0
        } else {
            1.0 - extinction_coeff + env.scattering / 8.0
        }
    } else {
        if out_idx == CENTER_IDX {
            1.0 / 16.0
        } else if out_idx != in_idx {
            env.scattering / 16.0
        } else {
            1.0 - extinction_coeff + env.scattering / 16.0
        }
    }
}
