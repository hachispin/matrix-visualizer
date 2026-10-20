//! Placeholder controls for the graph UI.

use three_d::{FrameInput, GUI, Viewport, egui};

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

    pub fn update(&mut self, gui: &mut GUI, frame: &mut FrameInput) -> Viewport {
        let mut graph_viewport = frame.viewport;

        gui.update(
            &mut frame.events,
            frame.accumulated_time,
            frame.viewport,
            frame.device_pixel_ratio,
            |root| {
                graph_viewport = self.show(root, frame.viewport, frame.device_pixel_ratio);
            },
        );

        graph_viewport
    }

    fn show(&mut self, ui: &mut egui::Ui, viewport: Viewport, pixel_ratio: f32) -> Viewport {
        egui::Panel::left("sidebar")
            .default_size(260.0)
            .size_range(180.0..=480.0)
            .show_inside(ui, |ui| {
                ui.heading("Hello world");
                ui.add(egui::Slider::new(&mut self.value, 0.0..=100.0).text("Value"));
                ui.label("Text");
                ui.text_edit_singleline(&mut self.text);
            });

        // The sidebar consumes the left part of the root UI. The graph gets the remainder.
        let left = (ui.available_rect_before_wrap().left() * pixel_ratio).round();
        let sidebar_width =
            ((left - viewport.x as f32).max(0.0) as u32).min(viewport.width.saturating_sub(1));
        Viewport {
            x: viewport.x
                + i32::try_from(sidebar_width).expect("Sidebar width exceeds window limits"),
            width: (viewport.width - sidebar_width).max(1),
            height: viewport.height.max(1),
            ..viewport
        }
    }
}
