use egui_plot::{Legend, Line, Plot, PlotPoints};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, ops::Deref, rc::Rc, sync::Arc};

use crate::{
    dbc::{Dbc, SerializableDbc},
    widgets::close_button_ui,
};

#[derive(Serialize, Deserialize, Default)]
pub struct AppSaveState {
    dbc: Option<SerializableDbc>,
}

pub struct App {
    pub dbc: Option<Dbc>,
    pub errors: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            dbc: None,
            errors: Vec::new(),
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            log::info!("Got saved data");
            App::from_save_state(eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default())
        } else {
            Default::default()
        }
    }

    pub fn handle_dbc(&mut self, name: String, bytes: Arc<[u8]>) {
        match Dbc::new(Arc::from(name), bytes) {
            Ok(dbc) => {
                let _ = self.dbc.insert(dbc);
            }
            Err(e) => self.errors.push(e),
        }
    }

    fn get_save_state(&self) -> AppSaveState {
        AppSaveState {
            dbc: self.dbc.as_ref().map(|dbc| dbc.into_serializable()),
        }
    }

    fn from_save_state(save_state: AppSaveState) -> Self {
        Self {
            dbc: save_state
                .dbc
                .map(|saved_dbc| Dbc::from_serializable(saved_dbc))
                .and_then(|dbc| dbc.ok()),
            ..Default::default()
        }
    }

    fn handle_file_inputs(&mut self, ctx: &egui::Context) {
        ctx.input(|input_state| {
            input_state
                .raw
                .dropped_files
                .iter()
                .filter(|file| file.name.to_lowercase().ends_with(".dbc"))
                .last()
                .map(|dbc_file| {
                    let bytes = dbc_file
                        .bytes
                        .clone()
                        .expect("Field is guranteed to be set by the backend");
                    self.handle_dbc(dbc_file.name.clone(), bytes);
                });
        });
    }
}

pub struct SharedApp(pub Rc<RefCell<App>>);

impl Deref for SharedApp {
    type Target = Rc<RefCell<App>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl eframe::App for SharedApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        log::info!("Saved data");
        eframe::set_value(
            storage,
            eframe::APP_KEY,
            &self.borrow_mut().get_save_state(),
        );
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut app = self.borrow_mut();

        app.handle_file_inputs(&ctx);

        egui::TopBottomPanel::top("top_panel").show(&ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        // Errors
        if !app.errors.is_empty() {
            let mut errors_to_close = Vec::new();
            egui::Window::new("The following errors ocurred").show(ctx, |ui| {
                app.errors
                    .iter()
                    .enumerate()
                    .for_each(|(error_idx, error)| {
                        ui.horizontal(|ui| {
                            ui.label(error);
                            if close_button_ui(ui, ui.max_rect()).clicked() {
                                errors_to_close.push(error_idx);
                            }
                        });
                        ui.separator();
                    });
            });

            errors_to_close.sort_unstable_by(|idx_a, idx_b| idx_b.cmp(idx_a));
            errors_to_close.iter().for_each(|error_idx| {
                app.errors.swap_remove(*error_idx);
            });
        }

        app.draw_side_panel(&ctx, self.clone());

        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.heading("Gráfica");
            Plot::new("KAKUKU")
                .legend(Legend::default())
                .show(ui, |plot_ui| {
                    plot_ui.line(Line::new(
                        "tusmuertos",
                        PlotPoints::new(vec![[0., 0.], [100., 100.]]),
                    ));
                });
        });
    }
}
