use egui_plot::{Legend, Line, Plot, PlotPoints};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    label: String,

    #[serde(skip)]
    value: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            value: 2.7,
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Cine");

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
