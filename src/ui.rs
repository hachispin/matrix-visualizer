//! Placeholder controls for the graph UI.

use three_d::egui;

#[derive(Default)]
pub struct Ui {
    value: f32,
    text: String,
}

impl Ui {
    pub fn new(context: &egui::Context) -> Self {
        context.set_theme(egui::Theme::Light);
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "IBM Plex Sans".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/IBMPlexSans-Regular.ttf")).into(),
        );

        for family in fonts.families.values_mut() {
            family.insert(0, "IBM Plex Sans".to_owned());
        }

        context.set_fonts(fonts);
        Self::default()
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Placeholder")
            .default_pos([16.0, 16.0])
            .resizable(false)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.heading("Hello world");
                ui.add(egui::Slider::new(&mut self.value, 0.0..=100.0).text("Value"));
                ui.label("Text");
                ui.text_edit_singleline(&mut self.text);
            });
    }
}
