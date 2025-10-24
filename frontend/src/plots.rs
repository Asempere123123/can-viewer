use egui::{Frame, Layout, Rect, Ui, UiBuilder};
use egui_plot::{Legend, Line, PlotPoints};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    App,
    dbc::{Dbc, Signal},
    messages::Messages,
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
                if plot.draw(plot_ui, idx, dbc, &app.messages) {
                    plots_to_close.push(idx);
                }
            }

            plots_to_close.sort_by(|a, b| b.cmp(a));
            for plot_to_close in plots_to_close {
                app.plots.0.remove(plot_to_close);
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

    fn draw(&mut self, ui: &mut Ui, number: usize, dbc: &Dbc, messages: &Messages) -> bool {
        let mut should_close = false;
        let (_, new_signal) = ui.dnd_drop_zone::<Signal, _>(Frame::new().inner_margin(5), |ui| {
            ui.heading(format!("Plot {}:", number + 1));
            let mut close_rect = ui.max_rect();
            close_rect.max.y = close_rect.min.y + 2.;
            should_close = widgets::close_button_ui(ui, close_rect).clicked();

            let max_rect = ui.max_rect();
            ui.horizontal(|ui| {
                self.draw_plot(ui, dbc, number, max_rect, messages);
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
                self.signals.remove(signal_to_erase);
            }
        });
    }

    fn draw_plot(
        &mut self,
        ui: &mut Ui,
        dbc: &Dbc,
        plot_idx: usize,
        max_rect: Rect,
        messages: &Messages,
    ) {
        // TODO: this is local to each plot. So if 2 plots are created, their start instant will not match
        // This might not be the expected behaviour by anyone
        let Some(first_signal_id) = self.signals.first().map(|signal| signal.message_id) else {
            // Still have to draw an empty one
            egui_plot::Plot::new(plot_idx)
                .height(max_rect.height() * 0.9)
                .width(max_rect.width() * 0.8)
                .legend(Legend::default())
                .show(ui, |_| {});
            return;
        };
        let Some(initial_timestamp) = messages
            .0
            .get(&first_signal_id)
            .map(|messages| {
                // TODO: Code will be incorrect starting on year 2262 since it will overflow
                messages.iter().next().map(|first_msg| unsafe {
                    first_msg.timestamp.timestamp_nanos_opt().unwrap_unchecked()
                })
            })
            .flatten()
        else {
            // Still have to draw an empty one
            egui_plot::Plot::new(plot_idx)
                .height(max_rect.height() * 0.9)
                .width(max_rect.width() * 0.8)
                .legend(Legend::default())
                .show(ui, |_| {});
            return;
        };

        // Cuidado con usar usizes para ids en otro lado que entonces hay colisiones
        egui_plot::Plot::new(plot_idx)
            .height(max_rect.height() * 0.9)
            .width(max_rect.width() * 0.8)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                // Esto no deberia existir, es una aberracion que de alguna forma funciona
                self.signals
                    .iter()
                    .map(|signal| {
                        (
                            signal.message_id,
                            &dbc.messages_map[&signal.message_id].signals()[signal.signal_idx],
                        )
                    })
                    .filter_map(|(message_id, signal)| {
                        messages.0.get(&message_id).map(move |messages| {
                            (
                                signal.name(),
                                messages.iter().map(move |recv_message| {
                                    let y = decode_signal(signal, &recv_message.contents);

                                    [
                                        // TODO: Same as before, change on year 2262
                                        (unsafe {
                                            recv_message
                                                .timestamp
                                                .timestamp_nanos_opt()
                                                .unwrap_unchecked()
                                        } - initial_timestamp)
                                            as f64
                                            / 10.0e9,
                                        y,
                                    ]
                                }),
                            )
                        })
                    })
                    .for_each(|(signal_name, positions)| {
                        plot_ui.line(Line::new(signal_name, PlotPoints::from_iter(positions)));
                    });
            });
    }
}

// https://docs.rs/can_decode/latest/src/can_decode/lib.rs.html#270-299
// Could be made faster but i wont (simd + remove bitwise loops)
fn decode_signal(signal_def: &can_dbc::Signal, data: &[u8]) -> f64 {
    // Get signal properties
    let start_bit = *signal_def.start_bit() as usize;
    let signal_size = *signal_def.signal_size() as usize;
    let byte_order = signal_def.byte_order();
    let value_type = signal_def.value_type();
    let factor = signal_def.factor();
    let offset = signal_def.offset();

    // Extract raw value based on byte order and signal properties
    let raw_value = extract_signal_value(data, start_bit, signal_size, *byte_order);

    // Convert to signed if needed
    let raw_value = if *value_type == can_dbc::ValueType::Signed {
        // Convert to signed based on signal size
        let max_unsigned = (1u64 << signal_size) - 1;
        let sign_bit = 1u64 << (signal_size - 1);

        if raw_value & sign_bit != 0 {
            // Negative number - extend sign
            (raw_value | (!max_unsigned)) as i64 as f64
        } else {
            raw_value as f64
        }
    } else {
        raw_value as f64
    };

    // Apply scaling
    let scaled_value = raw_value * factor + offset;

    scaled_value
}

fn extract_signal_value(
    data: &[u8],
    start_bit: usize,
    size: usize,
    byte_order: can_dbc::ByteOrder,
) -> u64 {
    let mut result = 0u64;

    match byte_order {
        can_dbc::ByteOrder::LittleEndian => {
            let start_byte = start_bit / 8;
            let start_bit_in_byte = start_bit % 8;

            let mut remaining_bits = size;
            let mut current_byte = start_byte;
            let mut bit_offset = start_bit_in_byte;

            while remaining_bits > 0 && current_byte < data.len() {
                let bits_in_this_byte = std::cmp::min(remaining_bits, 8 - bit_offset);
                let mask = ((1u64 << bits_in_this_byte) - 1) << bit_offset;
                let byte_value = ((data[current_byte] as u64) & mask) >> bit_offset;

                result |= byte_value << (size - remaining_bits);

                remaining_bits -= bits_in_this_byte;
                current_byte += 1;
                bit_offset = 0;
            }
        }
        can_dbc::ByteOrder::BigEndian => {
            // Idk if this is right
            let mut bit_pos = start_bit;

            for _ in 0..size {
                let byte_idx = bit_pos / 8;
                let bit_idx = 7 - (bit_pos % 8);

                if byte_idx >= data.len() {
                    break;
                }

                let bit_val = (data[byte_idx] >> bit_idx) & 1;
                result = (result << 1) | (bit_val as u64);

                bit_pos += 1;
            }
        }
    }

    result
}
