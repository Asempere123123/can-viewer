use egui_plot::{Legend, Line, Plot, PlotPoints};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::dbc::{Dbc, SerializableDbc};

#[derive(Serialize, Deserialize, Default)]
pub struct AppSaveState {
    dbc: Option<SerializableDbc>,
}

pub struct App {
    dbc: Option<Dbc>,
    errors: Vec<String>,
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

    fn handle_dbc(&mut self, name: String, bytes: Arc<[u8]>) {
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
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        log::info!("Saved data");
        eframe::set_value(storage, eframe::APP_KEY, &self.get_save_state());
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
            });

            if !self.errors.is_empty() {
                ui.label("cacaca");
                self.errors.iter().for_each(|error| {
                    ui.label(error);
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut should_remove_dbc = false;
            if let Some(dbc) = &self.dbc {
                ui.horizontal(|ui| {
                    ui.label(&*dbc.name);
                    should_remove_dbc = ui.button("Quitar").clicked();
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("No dbc loaded");
                    /*if ui.button("Seleccionar DBC").clicked() {}*/
                });
            }
            if should_remove_dbc {
                let _ = self.dbc.take();
            }

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
