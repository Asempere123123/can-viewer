use rfd::AsyncFileDialog;
use std::{cell::RefCell, rc::Rc, sync::Arc};
use wasm_bindgen_futures::spawn_local;

use crate::App;

impl App {
    pub fn draw_side_panel(&mut self, ctx: &egui::Context, app_handle: Rc<RefCell<App>>) {
        let ctx_clone = ctx.clone();
        egui::SidePanel::left("dbc_panel")
            .resizable(true)
            .show(ctx, |ui| {
                // File selector
                let mut should_remove_dbc = false;
                if let Some(dbc) = &self.dbc {
                    ui.horizontal(|ui| {
                        ui.label(&*dbc.name);
                        should_remove_dbc = ui.button("Quitar").clicked();
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("No dbc loaded");
                        if ui.button("Seleccionar DBC").clicked() {
                            spawn_local(async move {
                                if let Some(file) = AsyncFileDialog::new()
                                    .add_filter("DBC Files", &["dbc", "DBC"])
                                    .set_directory("/")
                                    .pick_file()
                                    .await
                                {
                                    app_handle
                                        .borrow_mut()
                                        .handle_dbc(file.file_name(), Arc::from(file.read().await));
                                    ctx_clone.request_repaint();
                                }
                            });
                        }
                    });
                }
                if should_remove_dbc {
                    let _ = self.dbc.take();
                }

                // Message viewer
                let Some(dbc) = &self.dbc else {
                    ui.heading("No DBC file loaded");
                    return;
                };

                ui.heading("DBC Messages");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for message in dbc.inner.messages() {
                        egui::collapsing_header::CollapsingHeader::new(message.message_name())
                            .show(ui, |ui| {
                                message.signals().iter().for_each(|signal| {
                                    ui.label(signal.name());
                                });
                            });
                    }
                });
            });
    }
}
