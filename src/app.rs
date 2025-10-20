use base64::{Engine, engine::general_purpose::URL_SAFE, write::EncoderStringWriter};
use eframe::Storage;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use std::{cell::RefCell, ops::Deref, rc::Rc, sync::Arc};

use crate::{
    dbc::{Dbc, SerializableDbc},
    messages::Messages,
    widgets::close_button_ui,
};

#[derive(Default)]
pub struct AppSaveState {
    dbc: Option<SerializableDbc>,
    messages: Messages,
}

impl AppSaveState {
    const DBC: &str = "DBC";
    const MESSAGES: &str = "MESSAGES";

    fn save(self, storage: &mut dyn Storage) {
        let mut writer = EncoderStringWriter::new(&URL_SAFE);
        bincode::serde::encode_into_std_write(&self.dbc, &mut writer, bincode::config::standard())
            .unwrap();
        // TODO: quitar este unwrap, es solo para que me avise al hacer pruebas
        storage.set_string(AppSaveState::DBC, writer.into_inner());

        let mut writer = EncoderStringWriter::new(&URL_SAFE);
        bincode::serde::encode_into_std_write(
            &self.messages,
            &mut writer,
            bincode::config::standard(),
        )
        .unwrap();
        // TODO: quitar este unwrap, es solo para que me avise al hacer pruebas
        storage.set_string(AppSaveState::MESSAGES, writer.into_inner());
    }

    fn load(storage: &dyn Storage) -> AppSaveState {
        let Some(b64_raw) = storage.get_string(AppSaveState::DBC) else {
            return Default::default();
        };
        let Ok(raw) = URL_SAFE.decode(&b64_raw) else {
            return Default::default();
        };
        let dbc = bincode::serde::decode_from_slice(&raw, bincode::config::standard())
            .map(|val| val.0)
            .unwrap_or_default();

        let Some(b64_raw) = storage.get_string(AppSaveState::MESSAGES) else {
            return Default::default();
        };
        let Ok(raw) = URL_SAFE.decode(&b64_raw) else {
            return Default::default();
        };
        let messages = bincode::serde::decode_from_slice(&raw, bincode::config::standard())
            .map(|val| val.0)
            .unwrap_or_default();

        AppSaveState { dbc, messages }
    }
}

pub struct App {
    pub dbc: Option<Dbc>,
    pub messages: Messages,
    pub errors: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            dbc: None,
            messages: Messages::empty(),
            errors: Vec::new(),
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            log::info!("Got saved data");
            App::from_save_state(AppSaveState::load(storage))
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
            messages: self.messages.clone(),
        }
    }

    fn from_save_state(save_state: AppSaveState) -> Self {
        Self {
            dbc: save_state
                .dbc
                .map(|saved_dbc| Dbc::from_serializable(saved_dbc))
                .and_then(|dbc| dbc.ok()),
            messages: save_state.messages,
            ..Default::default()
        }
    }

    fn handle_file_inputs(&mut self, ctx: &egui::Context) {
        ctx.input(|input_state| {
            input_state.raw.dropped_files.iter().for_each(|file| {
                let file_name = file.name.to_lowercase();
                let bytes = file
                    .bytes
                    .clone()
                    .expect("Field is guranteed to be set by the backend");

                if file_name.ends_with(".dbc") {
                    self.handle_dbc(file.name.clone(), bytes);
                } else if file_name.ends_with(".log") {
                    let file_contents = String::from_utf8_lossy(&bytes);

                    self.messages.extend(&Messages::from_string(file_contents));
                }
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
        self.borrow().get_save_state().save(storage);
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
