use egui::{Frame, Layout, Rect, Ui, UiBuilder};
use egui_plot::{Legend, Line, PlotPoints};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    App,
    dbc::{Dbc, Signal},
    widgets,
};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Plots(Vec<Plot>);

impl Plots {
    pub fn add_one(&mut self) {
        self.0.push(Plot::new());
    }

    pub fn draw(app: &mut App, ui: &mut Ui) {
        let Some(dbc) = &app.dbc else {
            ui.heading("No Dbc loaded");
            return;
        };

        if app.plots.0.is_empty() {
            ui.heading("Add a plot to start");
            return;
        }

        ui.vertical(|ui| {
            let total_height = ui.available_height();
            let n = app.plots.0.len();
            let each_height = total_height / n as f32;

            let mut plots_to_close = Vec::new();
            for (idx, plot) in app.plots.0.iter_mut().enumerate() {
                let rect = ui
                    .allocate_space(egui::vec2(ui.available_width(), each_height))
                    .1;

                let ui_builder = UiBuilder {
                    max_rect: Some(rect),
                    layout: Some(Layout::top_down(egui::Align::Min)),
                    ..UiBuilder::new()
                };
                let plot_ui = &mut ui.new_child(ui_builder);
                if plot.draw(plot_ui, idx, dbc) {
                    plots_to_close.push(idx);
                }
            }

            plots_to_close.sort_by(|a, b| b.cmp(a));
            for plot_to_close in plots_to_close {
                app.plots.0.swap_remove(plot_to_close);
            }
        });
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Plot {
    signals: Vec<Arc<Signal>>,
}

impl Plot {
    fn new() -> Self {
        Self {
            signals: Vec::new(),
        }
    }

    fn draw(&mut self, ui: &mut Ui, number: usize, dbc: &Dbc) -> bool {
        let mut should_close = false;
        let (_, new_signal) = ui.dnd_drop_zone::<Signal, _>(Frame::new().inner_margin(5), |ui| {
            ui.heading(format!("Plot {}:", number + 1));
            let mut close_rect = ui.max_rect();
            close_rect.max.y = close_rect.min.y + 2.;
            should_close = widgets::close_button_ui(ui, close_rect).clicked();

            let max_rect = ui.max_rect();
            ui.horizontal(|ui| {
                self.draw_plot(ui, dbc, number, max_rect);
                ui.separator();
                self.draw_list(ui, dbc);
            });
        });

        if let Some(new_signal) = new_signal {
            self.signals.push(new_signal);
        }

        should_close
    }

    fn draw_list(&mut self, ui: &mut Ui, dbc: &Dbc) {
        let mut signals_to_erase = Vec::new();
        ui.vertical(|ui| {
            for (signal_plot_storage_idx, signal) in self.signals.iter().enumerate() {
                let Some(message) = dbc.messages_map.get(&signal.message_id) else {
                    continue;
                };

                let signal = &message.signals()[signal.signal_idx];

                ui.horizontal(|ui| {
                    ui.label(format!("{} > {}", message.message_name(), signal.name()));
                    if widgets::close_button_ui(ui, ui.max_rect()).clicked() {
                        signals_to_erase.push(signal_plot_storage_idx);
                    }
                });
            }

            signals_to_erase.sort_by(|a, b| b.cmp(a));
            for signal_to_erase in signals_to_erase {
                self.signals.swap_remove(signal_to_erase);
            }
        });
    }

    fn draw_plot(&mut self, ui: &mut Ui, dbc: &Dbc, plot_idx: usize, max_rect: Rect) {
        // Cuidado con usar usizes para ids en otro lado que entonces hay colisiones
        egui_plot::Plot::new(plot_idx)
            .height(max_rect.height() * 0.9)
            .width(max_rect.width() * 0.8)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new(
                    "tusmuertos",
                    PlotPoints::new(vec![[0., 0.], [100., 100.]]),
                ));
            });
    }
}
