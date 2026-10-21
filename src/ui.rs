//! Sidebar editors for points and 3×3 linear transformations.

use three_d::{Event, FrameInput, GUI, Key, Mat3, SquareMatrix, Viewport, egui};

use crate::shapes::Point;

struct MatrixInput {
    id: usize,
    matrix: Mat3,
}

impl Default for MatrixInput {
    fn default() -> Self {
        Self {
            id: 0,
            matrix: Mat3::identity(),
        }
    }
}

#[derive(Clone, Copy)]
struct FocusedMatrixEntry {
    id: egui::Id,
    last: bool,
}

pub struct Ui {
    pub points: Vec<Point>,
    pub transformation: Mat3,
    matrices: Vec<MatrixInput>,
    focused_matrix_entry: Option<FocusedMatrixEntry>,
    error: Option<String>,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            points: vec![Point::default()],
            transformation: Mat3::identity(),
            matrices: vec![MatrixInput::default()],
            focused_matrix_entry: None,
            error: None,
        }
    }
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
        // go to the next entry if a comma is typed
        if let Some(entry) = self.focused_matrix_entry
            && gui.context().memory(egui::Memory::focused) == Some(entry.id)
        {
            let mut push_tab = false;

            frame.events.retain_mut(|event| {
                if matches!(event, Event::Text(text) if text == ",") {
                    // drop comma type event if on last
                    if entry.last {
                        return false;
                    }

                    *event = Event::KeyPress {
                        kind: Key::Tab,
                        modifiers: three_d::Modifiers::default(),
                        handled: false,
                    };

                    push_tab = true;
                }

                true
            });

            if push_tab {
                frame.events.push(Event::KeyRelease {
                    kind: Key::Tab,
                    modifiers: three_d::Modifiers::default(),
                    handled: false,
                });
            }
        }

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
            .default_size(300.0)
            .size_range(220.0..=480.0)
            .show_inside(ui, |ui| self.controls(ui));

        let left = (ui.available_rect_before_wrap().left() * pixel_ratio).round();

        let sidebar_width =
            ((left - viewport.x as f32).max(0.0) as u32).min(viewport.width.saturating_sub(1));

        Viewport {
            x: viewport.x
                + i32::try_from(sidebar_width).expect("Sidebar width exceeds window limits!"),
            width: (viewport.width - sidebar_width).max(1),
            height: viewport.height.max(1),
            ..viewport
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        self.focused_matrix_entry = None;

        ui.heading("Matrix Visualizer");

        ui.horizontal_wrapped(|ui| {
            if ui.button("Add a point").clicked() {
                let id = available_id(self.points.iter().map(|point| point.id));

                self.points.push(Point {
                    id,
                    ..Point::default()
                });

                self.points.sort_by_key(|point| point.id);
            }

            if ui.button("Add a matrix").clicked() {
                let id = available_id(self.matrices.iter().map(|matrix| matrix.id));

                self.matrices.push(MatrixInput {
                    id,
                    ..MatrixInput::default()
                });
                self.matrices.sort_by_key(|matrix| matrix.id);
            }
        });

        ui.separator();

        if ui.button("Reset transformations").clicked() {
            self.transformation = Mat3::identity();
            self.error = None;
        }

        if let Some(error) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }

        ui.separator();

        let mut apply = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.heading("Points");

                if self.points.is_empty() {
                    ui.label("Add a point and plot its coordinates.");
                }

                self.points.retain_mut(|point| {
                    ui.push_id(("point", point.id), |ui| {
                        Self::point_editor(ui, point, self.transformation)
                    })
                    .inner
                });

                ui.add_space(12.0);
                ui.heading("Matrices");

                self.matrices.retain_mut(|matrix| {
                    ui.push_id(("matrix", matrix.id), |ui| {
                        let (keep, transform) =
                            Self::matrix_editor(ui, matrix, &mut self.focused_matrix_entry);

                        if transform {
                            apply = Some(matrix.matrix);
                        }

                        keep
                    })
                    .inner
                });
            });

        if let Some(matrix) = apply {
            let candidate = matrix * self.transformation;
            let entries: &[[_; 3]; 3] = candidate.as_ref();

            if entries.iter().flatten().all(|value| value.is_finite()) {
                self.transformation = candidate;
                self.error = None;
            } else {
                self.error = Some("Produced values are too large to plot!".to_owned());
            }
        }
    }

    /// Returns false if the remove button is clicked.
    fn point_editor(ui: &mut egui::Ui, point: &mut Point, transformation: Mat3) -> bool {
        let mut remove = false;

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                let color = point.color();

                ui.colored_label(
                    egui::Color32::from_rgb(color.r, color.g, color.b),
                    format!("Point {}", point.id + 1),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    remove = ui.small_button("Remove").clicked();
                    ui.checkbox(&mut point.hidden, "Hide");
                });
            });

            let width = (ui.available_width() - 2.0 * ui.spacing().item_spacing.x) / 3.0;

            ui.horizontal(|ui| {
                for (axis, coordinate) in [
                    ("x", &mut point.position.x),
                    ("y", &mut point.position.y),
                    ("z", &mut point.position.z),
                ] {
                    ui.add_sized(
                        [width, 24.0],
                        number_input(coordinate).prefix(format!("{axis}: ")),
                    );
                }
            });

            let result = transformation * point.position;

            ui.small(format!(
                "Result: ({:.3}, {:.3}, {:.3})",
                result.x, result.y, result.z
            ));
        });

        ui.add_space(6.0);

        !remove
    }

    fn matrix_editor(
        ui: &mut egui::Ui,
        matrix: &mut MatrixInput,
        focused_entry: &mut Option<FocusedMatrixEntry>,
    ) -> (bool, bool) {
        let mut remove = false;
        let mut transform = false;

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong(format!("Matrix #{} (3×3)", matrix.id + 1));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    remove = ui.small_button("Remove").clicked();
                });
            });

            let width = (ui.available_width() - 2.0 * ui.spacing().item_spacing.x) / 3.0;

            egui::Grid::new("entries").num_columns(3).show(ui, |ui| {
                for row in 0..3 {
                    for column in 0..3 {
                        let response = ui.add_sized(
                            [width, 24.0],
                            number_input(&mut matrix.matrix[column][row]),
                        );

                        if response.has_focus() {
                            *focused_entry = Some(FocusedMatrixEntry {
                                id: response.id,
                                last: row == 2 && column == 2,
                            });
                        }
                    }

                    ui.end_row();
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Identity").clicked() {
                    matrix.matrix = Mat3::identity();
                }

                transform = ui.button("Transform").clicked();
            });
        });

        ui.add_space(6.0);

        (!remove, transform && !remove)
    }
}

fn available_id(ids: impl Iterator<Item = usize>) -> usize {
    let mut used: Vec<_> = ids.collect();
    used.sort_unstable();

    let mut available = 0;

    for id in used {
        if id > available {
            break;
        }

        if id == available {
            available += 1;
        }
    }

    available
}

fn number_input(value: &mut f32) -> egui::DragValue<'_> {
    egui::DragValue::new(value).speed(0.1).range(-1.0e6..=1.0e6)
}
