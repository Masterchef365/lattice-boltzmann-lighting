use egui::{
    CentralPanel, Color32, DragValue, Pos2, Rect, RichText, Scene, SidePanel, TopBottomPanel,
};
use egui_pixel_editor::{Brush, ImageEditor};
use glam::Vec3;
use sim::Sim;
mod sim;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(BoltzmannApp::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(BoltzmannApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

pub struct BoltzmannApp {
    pub sim: Sim,
    pub save_data: SaveData,
    pub scene_rect: Rect,
    pub light_source_editor: ImageEditor<sim::Cell>,
    pub light_editor: ImageEditor<sim::Cell>,
    pub env_editor: ImageEditor<sim::Environment>,
    pub edit_layer: EditLayer,
    pub n_step: usize,
    pub env_value: sim::Environment,
    pub reset_env_value: sim::Environment,
    pub cell_value: sim::Cell,
    pub run: bool,
    pub brush_size: isize,
    pub new_sim_dims: (usize, usize),
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum EditLayer {
    LightSource,
    #[default]
    Light,
    Environment,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct SaveData {
    example_value: f32,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            // Example stuff:
            example_value: 2.7,
        }
    }
}

impl BoltzmannApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let save_data = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default();

        let new_sim_dims @ (w, h) = (200, 100);
        let reset_env_value = sim::Environment {
            scattering: 1e-2,
            absorbtion: 0.0,
        };
        let sim = Sim::new(w, h, reset_env_value);

        let tile_texture_width = 512;

        Self {
            new_sim_dims,
            n_step: 1,
            save_data,
            sim,
            scene_rect: Rect::ZERO,
            light_source_editor: ImageEditor::from_tile_size(tile_texture_width),
            light_editor: ImageEditor::from_tile_size(tile_texture_width),
            env_editor: ImageEditor::from_tile_size(tile_texture_width),
            edit_layer: EditLayer::default(),
            env_value: sim::Environment {
                scattering: 1.0,
                absorbtion: 0.0,
            },
            reset_env_value,
            cell_value: sim::Cell {
                dirs: [
                    Vec3::new(0.5, 0., 1.),
                    Vec3::ZERO,
                    Vec3::ZERO,
                    Vec3::ZERO,
                    Vec3::ZERO,
                    Vec3::ZERO,
                    Vec3::ZERO,
                    Vec3::ZERO,
                    Vec3::ZERO,
                ],
            },
            run: true,
            brush_size: 0,
            //light_editor: ImageEditor::new(&cc.egui_ctx),
            //world_editor: ImageEditor::new(&cc.egui_ctx),
        }
    }
}

impl eframe::App for BoltzmannApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.save_data);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.run {
            ctx.request_repaint();
        }

        SidePanel::left("cfg").show(ctx, |ui| {
            ui.heading("Drawing");
            ui.group(|ui| {
                ui.strong("Current layer: ");
                ui.selectable_value(
                    &mut self.edit_layer,
                    EditLayer::LightSource,
                    "Light Sources",
                );
                ui.selectable_value(&mut self.edit_layer, EditLayer::Light, "Light");
                ui.selectable_value(&mut self.edit_layer, EditLayer::Environment, "Environment");
            });

            ui.group(|ui| {
                ui.strong("Brush");
                ui.add(
                    DragValue::new(&mut self.brush_size)
                        .prefix("Brush size: ")
                        .range(0..=isize::MAX),
                );

                match self.edit_layer {
                    EditLayer::Environment => {
                        ui.add(
                            DragValue::new(&mut self.env_value.scattering)
                                .prefix("Scattering: ")
                                .speed(1e-2),
                        );
                        ui.add(
                            DragValue::new(&mut self.env_value.absorbtion)
                                .prefix("Absorbtion: ")
                                .speed(1e-2),
                        );
                    }
                    EditLayer::Light | EditLayer::LightSource => {
                        egui::Grid::new("light").num_columns(3).show(ui, |ui| {
                            for row in self.cell_value.dirs.chunks_exact_mut(3) {
                                for value in row.iter_mut() {
                                    //ui.add(DragValue::new(value));
                                    let mut v = value.to_array();
                                    ui.color_edit_button_rgb(&mut v);
                                    [value.x, value.y, value.z] = v;
                                }
                                ui.end_row();
                            }
                        });
                    }
                }
            });

            ui.group(|ui| {
                ui.strong("Reset settings");
                ui.add(DragValue::new(&mut self.new_sim_dims.0).prefix("Width: "));
                ui.add(DragValue::new(&mut self.new_sim_dims.1).prefix("Height: "));
                ui.add(DragValue::new(&mut self.reset_env_value.scattering).prefix("Scattering: "));
            });
        });

        TopBottomPanel::bottom("Time").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut do_step = self.run;

                let text = if self.run { "Pause ⏸" } else { "Run ▶" };
                if ui.button(RichText::new(text).size(20.)).clicked() {
                    self.run = !self.run;
                }

                if ui.button(RichText::new("Reset light").size(20.)).clicked() {
                    self.sim.light.fill(sim::Cell::default());
                }

                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Step").size(20.)).clicked() {
                        do_step = true;
                    }
                    //ui.add(DragValue::new(&mut self.n_step));
                });
                if ui
                    .button(RichText::new("Reset everything").size(20.))
                    .clicked()
                {
                    let (w, h) = self.new_sim_dims;
                    self.sim = Sim::new(w, h, self.reset_env_value);
                    self.light_source_editor.force_image_update();
                    self.env_editor.force_image_update();
                    self.light_editor.force_image_update();
                }

                if do_step {
                    for _ in 0..self.n_step {
                        self.sim.step();
                    }
                    self.light_editor.force_image_update();
                }
            });
        });

        let brush = Brush::Rectangle(self.brush_size, self.brush_size);

        CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                Scene::new().zoom_range(0.1..=100.0).show(
                    ui,
                    &mut self.scene_rect,
                    |ui| match self.edit_layer {
                        EditLayer::Light => {
                            self.env_editor.draw(ui, &mut self.sim.env, Pos2::ZERO);
                            self.light_editor
                                .edit(ui, &mut self.sim.light, self.cell_value, brush);
                        }
                        EditLayer::LightSource => {
                            self.light_editor.draw(ui, &mut self.sim.light, Pos2::ZERO);
                            self.light_source_editor.edit(
                                ui,
                                &mut self.sim.light_source,
                                self.cell_value,
                                brush,
                            );
                            self.env_editor.draw(ui, &mut self.sim.env, Pos2::ZERO);
                        }
                        EditLayer::Environment => {
                            self.light_editor.draw(ui, &mut self.sim.light, Pos2::ZERO);
                            self.env_editor
                                .edit(ui, &mut self.sim.env, self.env_value, brush);
                        }
                    },
                );
            });
        });
    }
}
