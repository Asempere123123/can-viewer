use egui::Id;
use rfd::AsyncFileDialog;
use std::{cell::RefCell, rc::Rc, sync::Arc};
use wasm_bindgen_futures::spawn_local;

use crate::{App, dbc::Signal, messages::Messages};

impl App {
    pub fn draw_side_panel(&mut self, ctx: &egui::Context, app_handle: Rc<RefCell<App>>) {
        egui::SidePanel::left("dbc_panel")
            .resizable(true)
            .show(ctx, |ui| {
                // Loaded Message file selector
                ui.horizontal(|ui| {
                    ui.heading("Messages:");
                    if ui.button("Clear").clicked() {
                        self.messages.0.clear();
                    }
                    if ui.button("Add from log file").clicked() {
                        let app_handle = app_handle.clone();
                        let ctx = ctx.clone();
                        spawn_local(async move {
                            if let Some(file) = AsyncFileDialog::new()
                                .add_filter("log Files", &["log", "LOG"])
                                .set_directory("/")
                                .pick_file()
                                .await
                            {
                                let bytes = file.read().await;
                                let file_contents = String::from_utf8_lossy(&bytes);

                                app_handle
                                    .borrow_mut()
                                    .messages
                                    .extend(&Messages::from_string(file_contents));
                                ctx.request_repaint();
                            }
                        });
                    }
                });
                ui.label(format!("{}", self.messages.len()));
                ui.separator();

                // Dbc File selector
                ui.heading("DBC:");
                let mut should_remove_dbc = false;
                if let Some(dbc) = &self.dbc {
                    ui.horizontal(|ui| {
                        ui.label(&*dbc.name);
                        should_remove_dbc = ui.button("Remove").clicked();
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("No dbc loaded");
                        let ctx = ctx.clone();
                        if ui.button("Select DBC").clicked() {
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
                                    ctx.request_repaint();
                                }
                            });
                        }
                    });
                }
                if should_remove_dbc {
                    let _ = self.dbc.take();
                }

                // DBC Message viewer
                let Some(dbc) = &self.dbc else {
                    ui.heading("No DBC file loaded");
                    return;
                };

                ui.heading("DBC Messages");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for message in dbc.inner.messages() {
                        egui::collapsing_header::CollapsingHeader::new(message.message_name())
                            .show(ui, |ui| {
                                message.signals().iter().enumerate().for_each(
                                    |(signal_idx, signal)| {
                                        ui.dnd_drag_source(
                                            Id::new(message.message_id().raw()).with(signal_idx),
                                            Signal {
                                                message_id: message.message_id().raw(),
                                                signal_idx,
                                            },
                                            |ui| {
                                                ui.label(signal.name());
                                            },
                                        );
                                    },
                                );
                            });
                    }
                });
            });
    }
}
