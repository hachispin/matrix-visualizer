//! Sidebar editors for points and 3×3 linear transformations.

use std::collections::VecDeque;

use three_d::{
    Event,
    FrameInput,
    GUI,
    Key,
    Mat3,
    MouseButton,
    PhysicalPoint,
    SquareMatrix,
    Vec3,
    Viewport,
    degrees,
    egui,
};

use crate::{
    polygons::{Polygon, Vertex},
    shapes::{LineSegment, Point},
};

const TRANSFORMATION_DURATION: f64 = 0.6;

struct TransformationAnimation {
    from: Mat3,
    to: Mat3,
    started_at: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MatrixPreset {
    Identity,
    Scale,
    RotateX,
    RotateY,
    RotateZ,
    Custom,
}

impl MatrixPreset {
    fn label(self) -> &'static str {
        match self {
            Self::Identity => "Identity",
            Self::Scale => "Scale",
            Self::RotateX => "Rotate about X",
            Self::RotateY => "Rotate about Y",
            Self::RotateZ => "Rotate about Z",
            Self::Custom => "Custom",
        }
    }
}

struct MatrixInput {
    id: usize,
    matrix: Mat3,
    preset: MatrixPreset,
    scale: f32,
    angle_degrees: f32,
}

impl Default for MatrixInput {
    fn default() -> Self {
        Self {
            id: 0,
            matrix: Mat3::identity(),
            preset: MatrixPreset::Identity,
            scale: 1.0,
            angle_degrees: 90.0,
        }
    }
}

impl MatrixInput {
    fn rebuild_preset(&mut self) {
        let angle = self.angle_degrees.rem_euclid(360.0);

        self.matrix = match self.preset {
            MatrixPreset::Identity => Mat3::identity(),
            MatrixPreset::Scale => {
                Mat3::from_diagonal(Vec3::new(self.scale, self.scale, self.scale))
            },
            MatrixPreset::RotateX => Mat3::from_angle_x(degrees(angle)),
            MatrixPreset::RotateY => Mat3::from_angle_y(degrees(angle)),
            MatrixPreset::RotateZ => Mat3::from_angle_z(degrees(angle)),
            MatrixPreset::Custom => return,
        };

        // Exact quarter turns have only zeroes and unit entries.
        if matches!(
            self.preset,
            MatrixPreset::RotateX | MatrixPreset::RotateY | MatrixPreset::RotateZ
        ) && angle % 90.0 == 0.0
        {
            for column in 0..3 {
                for row in 0..3 {
                    let value = self.matrix[column][row].round();
                    self.matrix[column][row] = if value == 0.0 { 0.0 } else { value };
                }
            }
        }
    }

    fn preset_selector(&mut self, ui: &mut egui::Ui) {
        let previous = self.preset;

        ui.horizontal_wrapped(|ui| {
            ui.label("Presets");

            egui::ComboBox::from_id_salt("preset")
                .selected_text(self.preset.label())
                .width(130.0)
                .show_ui(ui, |ui| {
                    for preset in [
                        MatrixPreset::Identity,
                        MatrixPreset::Scale,
                        MatrixPreset::RotateX,
                        MatrixPreset::RotateY,
                        MatrixPreset::RotateZ,
                        MatrixPreset::Custom,
                    ] {
                        ui.selectable_value(&mut self.preset, preset, preset.label());
                    }
                });
        });

        if self.preset != previous {
            self.rebuild_preset();
        }
    }

    fn preset_parameter(&mut self, ui: &mut egui::Ui) {
        let parameter = match self.preset {
            MatrixPreset::Scale => Some(("Scale factor", &mut self.scale)),
            MatrixPreset::RotateX | MatrixPreset::RotateY | MatrixPreset::RotateZ => {
                Some(("Degrees", &mut self.angle_degrees))
            },
            MatrixPreset::Identity | MatrixPreset::Custom => None,
        };

        if let Some((label, value)) = parameter {
            let changed = ui
                .push_id("preset_parameter", |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(label);

                        ui.add_sized([90.0, 24.0], number_input(value, ""))
                            .changed()
                    })
                    .inner
                })
                .inner;

            if changed {
                self.rebuild_preset();
            }
        }
    }
}

enum MatrixAction {
    Remove,
    Apply,
    CreateInverse,
}

#[derive(Clone, Copy)]
struct FocusedMatrixEntry {
    id: egui::Id,
    last: bool,
}

pub struct Ui {
    pub points: Vec<Point>,
    pub segments: Vec<LineSegment>,
    pub polygons: Vec<Polygon>,
    pub transformation: Mat3,
    animations: VecDeque<TransformationAnimation>,
    matrices: Vec<MatrixInput>,
    focused_matrix_entry: Option<FocusedMatrixEntry>,
    sidebar_dragging: bool,
    error: Option<String>,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            segments: Vec::new(),
            polygons: Vec::new(),
            transformation: Mat3::identity(),
            animations: VecDeque::new(),
            matrices: Vec::new(),
            focused_matrix_entry: None,
            sidebar_dragging: false,
            error: None,
        }
    }
}

impl Ui {
    pub fn new(context: &egui::Context) -> Self {
        context.set_theme(egui::Theme::Light);
        context.global_style_mut(|style| style.interaction.selectable_labels = false);

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
        self.advance_animation(frame.accumulated_time * 0.001);

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

        #[cfg(target_arch = "wasm32")]
        crate::web_input::begin_frame(
            frame.viewport.width as f32 / frame.device_pixel_ratio,
            frame.viewport.height as f32 / frame.device_pixel_ratio,
        );

        #[cfg(target_arch = "wasm32")]
        crate::web_input::prepare_events(&mut frame.events);

        gui.update(
            &mut frame.events,
            frame.accumulated_time,
            frame.viewport,
            frame.device_pixel_ratio,
            |root| {
                graph_viewport = self.show(root, frame.viewport, frame.device_pixel_ratio);
            },
        );

        #[cfg(target_arch = "wasm32")]
        crate::web_input::end_frame();

        self.capture_sidebar_input(&mut frame.events, graph_viewport);

        graph_viewport
    }

    fn capture_sidebar_input(&mut self, events: &mut [Event], viewport: Viewport) {
        let inside_plot = |position: PhysicalPoint| {
            position.x >= viewport.x as f32
                && position.x < viewport.x as f32 + viewport.width as f32
                && position.y >= viewport.y as f32
                && position.y < viewport.y as f32 + viewport.height as f32
        };

        for event in events {
            match event {
                Event::MousePress {
                    button: MouseButton::Left,
                    position,
                    handled,
                    ..
                } => {
                    self.sidebar_dragging = *handled || !inside_plot(*position);
                    *handled |= self.sidebar_dragging;
                },
                Event::MouseRelease {
                    button: MouseButton::Left,
                    position,
                    handled,
                    ..
                } => {
                    *handled |= self.sidebar_dragging || !inside_plot(*position);
                    self.sidebar_dragging = false;
                },
                Event::MouseMotion {
                    position, handled, ..
                }
                | Event::MouseWheel {
                    position, handled, ..
                }
                | Event::PinchGesture {
                    position, handled, ..
                }
                | Event::RotationGesture {
                    position, handled, ..
                } => {
                    *handled |= self.sidebar_dragging || !inside_plot(*position);
                },
                _ => {},
            }
        }
    }

    fn target_transformation(&self) -> Mat3 {
        self.animations
            .back()
            .map_or(self.transformation, |animation| animation.to)
    }

    pub fn animation_endpoints(&self) -> Option<(Mat3, Mat3)> {
        self.animations
            .front()
            .map(|animation| (animation.from, animation.to))
    }

    fn queue_transformation(&mut self, to: Mat3, time: f64) {
        let (from, started_at) = self
            .animations
            .back()
            .map_or((self.transformation, time), |animation| {
                (animation.to, animation.started_at + TRANSFORMATION_DURATION)
            });

        if from != to {
            self.animations.push_back(TransformationAnimation {
                from,
                to,
                started_at,
            });
        }
    }

    fn advance_animation(&mut self, time: f64) {
        while let Some(animation) = self.animations.front() {
            let progress =
                ((time - animation.started_at) / TRANSFORMATION_DURATION).clamp(0.0, 1.0);

            if progress >= 1.0 {
                self.transformation = animation.to;
                self.animations.pop_front();

                continue;
            }

            let eased = progress * progress * (3.0 - 2.0 * progress);

            for column in 0..3 {
                for row in 0..3 {
                    let from = f64::from(animation.from[column][row]);
                    let to = f64::from(animation.to[column][row]);

                    // Widen before interpolation so opposite large values cannot overflow.
                    self.transformation[column][row] = (from + (to - from) * eased) as f32;
                }
            }

            break;
        }
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

    fn point_controls(&mut self, ui: &mut egui::Ui, target: Mat3) {
        ui.horizontal_wrapped(|ui| {
            ui.heading("Points");

            if ui.button("Add a point").clicked() {
                let id = available_id(self.points.iter().map(|point| point.id));

                self.points.push(Point {
                    id,
                    ..Point::default()
                });

                self.points.sort_by_key(|point| point.id);
            }
        });

        if self.points.is_empty() {
            ui.label("Add a point and plot its coordinates.");
        }

        self.points.retain_mut(|point| {
            ui.push_id(("point", point.id), |ui| {
                Self::point_editor(ui, point, target)
            })
            .inner
        });
    }

    fn segment_controls(&mut self, ui: &mut egui::Ui, target: Mat3) {
        ui.horizontal_wrapped(|ui| {
            ui.heading("Line segments");

            if ui.button("Add a line segment").clicked() {
                let id = available_id(self.segments.iter().map(|segment| segment.id));

                self.segments.push(LineSegment {
                    id,
                    ..LineSegment::default()
                });
                self.segments.sort_by_key(|segment| segment.id);
            }
        });

        if self.segments.is_empty() {
            ui.label("Add a line segment and edit its two endpoints.");
        }

        self.segments.retain_mut(|segment| {
            ui.push_id(("segment", segment.id), |ui| {
                Self::segment_editor(ui, segment, target)
            })
            .inner
        });
    }

    fn polygon_controls(&mut self, ui: &mut egui::Ui, target: Mat3) {
        ui.heading("Polygons");

        ui.horizontal_wrapped(|ui| {
            let add_polygon = ui.button("Add a polygon").clicked();
            let add_cube = ui.button("Add unit cube").clicked();

            if add_polygon || add_cube {
                let id = available_id(self.polygons.iter().map(|polygon| polygon.id));
                let mut polygon = Polygon {
                    id,
                    ..Polygon::default()
                };

                if add_cube {
                    polygon.make_unit_cube();
                }

                self.polygons.push(polygon);
                self.polygons.sort_by_key(|polygon| polygon.id);
            }
        });

        if self.polygons.is_empty() {
            ui.label("Add a polygon to connect and shade its vertices.");
        }

        self.polygons.retain_mut(|polygon| {
            ui.push_id(("polygon", polygon.id), |ui| {
                Self::polygon_editor(ui, polygon, target)
            })
            .inner
        });
    }

    fn matrix_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.heading("Matrices");

            if ui.button("Add a matrix").clicked() {
                let id = available_id(self.matrices.iter().map(|matrix| matrix.id));

                self.matrices.push(MatrixInput {
                    id,
                    ..MatrixInput::default()
                });
                self.matrices.sort_by_key(|matrix| matrix.id);
            }
        });

        if self.matrices.is_empty() {
            ui.label("Add a matrix and apply it to transform your shapes.");
        }

        if ui.button("Reset transformations").clicked() {
            self.animations.clear();
            self.queue_transformation(Mat3::identity(), ui.input(|input| input.time));
            self.error = None;
        }

        if let Some(error) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        self.focused_matrix_entry = None;

        ui.heading("Matrix Visualizer");
        ui.separator();

        let mut apply = None;
        let mut create_inverse = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let target = self.target_transformation();

                self.point_controls(ui, target);

                ui.add_space(12.0);
                ui.separator();
                self.segment_controls(ui, target);

                ui.add_space(12.0);
                ui.separator();
                self.polygon_controls(ui, target);

                ui.add_space(12.0);
                ui.separator();
                self.matrix_buttons(ui);

                self.matrices.retain_mut(|matrix| {
                    ui.push_id(("matrix", matrix.id), |ui| {
                        match Self::matrix_editor(ui, matrix, &mut self.focused_matrix_entry) {
                            Some(MatrixAction::Remove) => false,
                            Some(MatrixAction::Apply) => {
                                apply = Some(matrix.matrix);

                                true
                            },
                            Some(MatrixAction::CreateInverse) => {
                                create_inverse = Some(matrix.matrix);

                                true
                            },
                            None => true,
                        }
                    })
                    .inner
                });
            });

        if let Some(matrix) = create_inverse {
            let inverse = matrix.invert().filter(|inverse| {
                let entries: &[[_; 3]; 3] = inverse.as_ref();

                entries.iter().flatten().all(|value| value.is_finite())
            });

            if let Some(mut matrix) = inverse {
                let entries: &mut [[_; 3]; 3] = matrix.as_mut();

                for value in entries.iter_mut().flatten() {
                    if *value == 0.0 {
                        *value = 0.0;
                    }
                }

                let id = available_id(self.matrices.iter().map(|matrix| matrix.id));

                self.matrices.push(MatrixInput {
                    id,
                    matrix,
                    preset: MatrixPreset::Custom,
                    ..MatrixInput::default()
                });
                self.matrices.sort_by_key(|matrix| matrix.id);
                self.error = None;
            } else {
                self.error = Some("Matrix has no finite inverse!".to_owned());
            }
        }

        if let Some(matrix) = apply {
            let candidate = matrix * self.target_transformation();
            let entries: &[[_; 3]; 3] = candidate.as_ref();
            let finite =
                entries.iter().flatten().all(|value| value.is_finite())
                    && self
                        .points
                        .iter()
                        .map(|point| point.position)
                        .chain(
                            self.segments
                                .iter()
                                .flat_map(|segment| [segment.endpoints.0, segment.endpoints.1]),
                        )
                        .chain(self.polygons.iter().flat_map(|polygon| {
                            polygon.vertices.iter().map(|vertex| vertex.position)
                        }))
                        .all(|position| {
                            let image = candidate * position;

                            [image.x, image.y, image.z].into_iter().all(f32::is_finite)
                        });

            if finite {
                self.queue_transformation(candidate, ui.input(|input| input.time));
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
                    remove = ui.button("Remove").clicked();

                    ui.checkbox(&mut point.hidden, "Hide");
                });
            });

            coordinate_editor(ui, &mut point.position, transformation);
        });

        ui.add_space(6.0);

        !remove
    }

    fn segment_editor(ui: &mut egui::Ui, segment: &mut LineSegment, transformation: Mat3) -> bool {
        let mut remove = false;

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                let color = segment.color();

                ui.colored_label(
                    egui::Color32::from_rgb(color.r, color.g, color.b),
                    format!("Segment {}", segment.id + 1),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    remove = ui.button("Remove").clicked();

                    ui.checkbox(&mut segment.hidden, "Hide");
                });
            });

            for (label, position) in [
                ("Start", &mut segment.endpoints.0),
                ("End", &mut segment.endpoints.1),
            ] {
                ui.push_id(label, |ui| {
                    ui.label(label);
                    coordinate_editor(ui, position, transformation);
                });

                ui.add_space(4.0);
            }
        });

        ui.add_space(6.0);

        !remove
    }

    fn polygon_editor(ui: &mut egui::Ui, polygon: &mut Polygon, transformation: Mat3) -> bool {
        let mut remove = false;

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                let color = polygon.color();

                ui.colored_label(
                    egui::Color32::from_rgb(color.r, color.g, color.b),
                    format!("Polygon {}", polygon.id + 1),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    remove = ui.button("Remove").clicked();

                    ui.checkbox(&mut polygon.hidden, "Hide");
                });
            });

            let sequential = polygon.faces.is_empty();
            let mut remove_vertex = None;

            for (index, vertex) in polygon.vertices.iter_mut().enumerate() {
                ui.push_id(vertex.id, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("Vertex {}", index + 1));

                        if sequential && index >= 3 {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Remove vertex").clicked() {
                                        remove_vertex = Some(index);
                                    }
                                },
                            );
                        }
                    });

                    coordinate_editor(ui, &mut vertex.position, transformation);
                });

                ui.add_space(4.0);
            }

            if let Some(index) = remove_vertex {
                polygon.vertices.remove(index);
            }

            if sequential {
                ui.label("Vertices connect in order, then back to the first.");

                if ui.button("Add vertex").clicked() {
                    let id = available_id(polygon.vertices.iter().map(|vertex| vertex.id));
                    let first = polygon.vertices[0].position;
                    let last = polygon.vertices[polygon.vertices.len() - 1].position;
                    let position = Vec3::from(std::array::from_fn::<_, 3, _>(|axis| {
                        first[axis].midpoint(last[axis])
                    }));

                    polygon.vertices.push(Vertex { id, position });
                }
            } else {
                ui.label("Eight corners joined by six shaded faces.");
            }
        });

        ui.add_space(6.0);

        !remove
    }

    fn matrix_editor(
        ui: &mut egui::Ui,
        matrix: &mut MatrixInput,
        focused_entry: &mut Option<FocusedMatrixEntry>,
    ) -> Option<MatrixAction> {
        let mut remove = false;
        let mut action = None;

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong(format!("Matrix #{} (3×3)", matrix.id + 1));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    remove = ui.button("Remove").clicked();
                });
            });

            matrix.preset_selector(ui);

            let width = (ui.available_width() - 2.0 * ui.spacing().item_spacing.x) / 3.0;
            let mut edited = false;

            egui::Grid::new("entries").num_columns(3).show(ui, |ui| {
                for row in 0..3 {
                    for column in 0..3 {
                        let response = ui.add_sized(
                            [width, 24.0],
                            number_input(&mut matrix.matrix[column][row], ""),
                        );
                        edited |= response.changed();

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

            if edited {
                matrix.preset = MatrixPreset::Custom;
            }

            matrix.preset_parameter(ui);

            ui.horizontal_wrapped(|ui| {
                if ui.button("Apply").clicked() {
                    action = Some(MatrixAction::Apply);
                }

                if ui.button("Create inverse").clicked() {
                    action = Some(MatrixAction::CreateInverse);
                }
            });
        });

        ui.add_space(6.0);

        if remove {
            Some(MatrixAction::Remove)
        } else {
            action
        }
    }
}

fn coordinate_editor(ui: &mut egui::Ui, position: &mut Vec3, transformation: Mat3) {
    let width = (ui.available_width() - 2.0 * ui.spacing().item_spacing.x) / 3.0;

    ui.horizontal(|ui| {
        for (axis, coordinate) in [
            ("x", &mut position.x),
            ("y", &mut position.y),
            ("z", &mut position.z),
        ] {
            ui.add_sized(
                [width, 24.0],
                number_input(coordinate, &format!("{axis}: ")),
            );
        }
    });

    let mut result = transformation * *position;

    ui.add_enabled_ui(false, |ui| {
        ui.horizontal(|ui| {
            for (axis, coordinate) in [
                ("x′", &mut result.x),
                ("y′", &mut result.y),
                ("z′", &mut result.z),
            ] {
                ui.add_sized(
                    [width, 24.0],
                    number_input(coordinate, &format!("{axis}: ")),
                );
            }
        });
    });
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

fn number_input<'a>(value: &'a mut f32, prefix: &'a str) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| {
        let id = ui.next_auto_id();
        let edit_id = id.with("number_text");

        #[cfg(target_arch = "wasm32")]
        let web_id = id.value().to_string();

        #[cfg(target_arch = "wasm32")]
        let committed = crate::web_input::take_value(&web_id)
            .and_then(|text| parse_number(&text))
            .is_some_and(|parsed| {
                *value = parsed;

                true
            });

        #[cfg(target_arch = "wasm32")]
        let touch_editing = crate::web_input::is_editing(&web_id);

        #[cfg(not(target_arch = "wasm32"))]
        let touch_editing = false;

        if touch_editing {
            ui.memory_mut(|memory| memory.surrender_focus(id));
        }

        let buffer = ui.data_mut(|data| data.remove_temp::<String>(edit_id));
        let editing = ui.is_enabled()
            && !touch_editing
            && (ui.memory(|memory| memory.has_focus(id)) || buffer.is_some());

        let (response, truncated, full_text) = if editing {
            number_editor(ui, id, value, buffer)
        } else {
            number_display(ui, *value, prefix, touch_editing)
        };

        #[cfg(target_arch = "wasm32")]
        let response = {
            let mut response = response;

            if committed {
                response.mark_changed();
            }

            if ui.is_enabled()
                && (touch_editing || ui.is_rect_visible(response.rect))
                && !egui::Popup::is_any_open(ui.ctx())
            {
                let rect = response.rect;
                let clip = ui.clip_rect();

                crate::web_input::register_number(
                    &web_id,
                    &value.to_string(),
                    prefix.trim_end_matches(": "),
                    &[
                        rect.left(),
                        rect.top(),
                        rect.width(),
                        rect.height(),
                        clip.left(),
                        clip.top(),
                        clip.right(),
                        clip.bottom(),
                    ],
                );
            }

            response
        };

        if truncated {
            response
                .on_hover_text(full_text.clone())
                .on_disabled_hover_text(full_text)
        } else {
            response
        }
    }
}

fn number_editor(
    ui: &mut egui::Ui,
    id: egui::Id,
    value: &mut f32,
    buffer: Option<String>,
) -> (egui::Response, bool, String) {
    let edit_id = id.with("number_text");
    let text_style = ui.style().drag_value_text_style.clone();
    let start_editing = buffer.is_none();
    let mut text = buffer.unwrap_or_else(|| value.to_string());

    if start_editing {
        let mut state = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::default(),
                egui::text::CCursor::new(text.chars().count()),
            )));
        state.store(ui.ctx(), id);
    }

    let output = egui::TextEdit::singleline(&mut text)
        .id(id)
        .clip_text(true)
        .font(text_style)
        .margin(ui.spacing().button_padding)
        .desired_width(ui.available_width())
        .min_size(ui.spacing().interact_size)
        .show(ui);

    let truncated = output.galley.size().x > output.text_clip_rect.width();
    let mut response = output.response.response;
    let finished = !response.has_focus() || response.clicked_elsewhere();
    let cancelled = ui.input(|input| input.key_pressed(egui::Key::Escape));

    if !cancelled && (response.changed() || finished) {
        let normalized: String = text
            .chars()
            .filter(|character| !character.is_whitespace())
            .map(|character| if character == '−' { '-' } else { character })
            .collect();

        if let Ok(parsed) = normalized.parse::<f32>() {
            if parsed.is_finite() {
                if finished {
                    *value = parsed;
                    response.mark_changed();
                }
            } else {
                text = value.to_string();
            }
        }
    }

    if finished {
        ui.memory_mut(|memory| memory.surrender_focus(id));
    } else {
        ui.data_mut(|data| data.insert_temp(edit_id, text.clone()));
    }

    (response, truncated, text)
}

fn number_display(
    ui: &mut egui::Ui,
    value: f32,
    prefix: &str,
    touch_editing: bool,
) -> (egui::Response, bool, String) {
    let text_style = ui.style().drag_value_text_style.clone();
    let font = text_style.resolve(ui.style());
    let color = ui.visuals().text_color();
    let width = ui.available_width() - 2.0 * ui.spacing().button_padding.x;

    // Only shorten the label, never the stored number or edit buffer.
    let mut label = value.to_string();

    let truncated = ui.fonts_mut(|fonts| {
        let mut fits = |text: &str| {
            fonts
                .layout_no_wrap(format!("{prefix}{text}"), font.clone(), color)
                .size()
                .x
                <= width
        };

        let truncated = !fits(&label);

        if truncated {
            while !label.is_empty() {
                label.pop();

                if fits(&format!("{label}…")) {
                    break;
                }
            }

            label.push('…');
        }

        truncated
    });

    let response = ui.add(
        egui::Button::new(egui::RichText::new(format!("{prefix}{label}")).text_style(text_style))
            .sense(egui::Sense::click())
            .min_size(ui.spacing().interact_size),
    );

    if response.clicked() && !touch_editing {
        ui.memory_mut(|memory| memory.request_focus(response.id));
    }

    (response, truncated, value.to_string())
}

#[cfg(target_arch = "wasm32")]
fn parse_number(text: &str) -> Option<f32> {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .map(|character| if character == '−' { '-' } else { character })
        .collect::<String>()
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite())
}
