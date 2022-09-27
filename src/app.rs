use std::borrow::{Borrow, BorrowMut};
use egui::mutex::Mutex;
use ndarray::Array2;
use crate::life;
use crate::life::Shift;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GameOfLife {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,

    #[serde(skip)]
    board: Mutex<Array2<bool>>,

    #[serde(skip)]
    continuous_play: bool,
}

impl Default for GameOfLife {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            board: Mutex::new(life::new_blank_board()),
            continuous_play: false,
        }
    }
}

impl GameOfLife {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for GameOfLife {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // println!("{:?}", *self.board.lock());

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            if ui.button("Import File").clicked() {
                *self.board.lock() = life::import_from_file();
            }

            if ui.button("Import RLE file").clicked() {
                *self.board.lock() = life::import_rle();
            }

            if ui.button("Next generation").clicked() || self.continuous_play {
                life::update_board(self.board.lock().borrow_mut());
                ctx.request_repaint();
            }

            ui.checkbox(&mut self.continuous_play, "Play continuously");

            if ui.button("Clear").clicked() {
                *self.board.lock() = life::new_blank_board();
            }

            if ui.button("New checkerboard").clicked() {
                *self.board.lock() = life::new_checkerboard();
            }

            ui.horizontal(|ui| {
                if ui.button("Up").clicked() {
                    life::shift_board(self.board.lock().borrow_mut(), Shift::Up(1));
                }
                if ui.button("Down").clicked() {
                    life::shift_board(self.board.lock().borrow_mut(), Shift::Down(1));
                }
                if ui.button("Left").clicked() {
                    life::shift_board(self.board.lock().borrow_mut(), Shift::Left(1));
                }
                if ui.button("Right").clicked() {
                    life::shift_board(self.board.lock().borrow_mut(), Shift::Right(1));
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let egui::InnerResponse { response, inner: top_left } =
                egui::Frame::canvas(&*ctx.style())
                    .show(ui, |ui| {
                        ui.set_width(life::DISPLAY_SIZE);
                        ui.set_height(life::DISPLAY_SIZE);

                        life::draw_board(ui, self.board.lock().borrow());

                        ui.min_rect().min
                    });

            let response = response.interact(egui::Sense::click());

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    life::edit_board(
                        self.board.lock().borrow_mut(),
                        pos.to_vec2() - top_left.to_vec2(),
                    );
                }
            }
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

