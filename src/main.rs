#![warn(
    clippy::pedantic,
    // nursery lints:
    clippy::use_self,
    clippy::or_fun_call,
    clippy::redundant_clone,
    clippy::equatable_if_let,
    clippy::needless_collect,
    // restriction lints:
    clippy::redundant_type_annotations,
    clippy::semicolon_inside_block,
    clippy::allow_attributes
)]
#![allow(
    clippy::enum_glob_use,
    reason = "when used, scope is restricted (e.g., inside functions)"
)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "many (valid) calculations trigger this, making it more noisy than helpful"
)]

mod grid;
mod labels;
mod palette;
mod shapes;
mod ui;
mod utils;

use std::iter;

use anyhow::Result;
use three_d::{
    Camera,
    ClearState,
    ColorMaterial,
    CpuMesh,
    Event,
    FrameOutput,
    GUI,
    Gm,
    Key,
    Mesh,
    MouseButton,
    Window,
    WindowSettings,
    prelude::*,
};

use crate::labels::Labels;

// this prevents drift (from repeatedly multiplying and dividing by 1.1)
fn zoom_magnification(steps: i32) -> f32 { 1.1_f32.powi(steps).clamp(0.001, 1000.0) }

fn input_handler(
    events: &mut [Event],
    camera: &mut Camera,
    centre: &mut Vec3,
    zoom_steps: &mut i32,
    keyboard_captured: bool,
    pointer_captured: bool,
) {
    // rotate the camera around the graph on drag
    for event in events {
        const DRAG_MULTIPLIER: f32 = 0.005;

        // pattern match this event
        if let Event::MouseMotion {
            button: Some(MouseButton::Left),
            delta,
            handled,
            ..
        } = event
        {
            if *handled || pointer_captured {
                continue;
            }

            camera.rotate_around_with_fixed_up(
                Vec3::zero(),
                DRAG_MULTIPLIER * delta.0,
                DRAG_MULTIPLIER * delta.1,
            );

            // prevent other event handlers from using this event again
            *handled = true;
        }

        if let Event::MouseWheel { delta, handled, .. } = event {
            if *handled || pointer_captured {
                continue;
            }

            // vertical scroll amount (positive = up)
            // don't change zoom steps if already at bounds
            if delta.1 > 0.0 && zoom_magnification(*zoom_steps) < 1000.0 {
                *zoom_steps += 1;
            } else if delta.1 < 0.0 && zoom_magnification(*zoom_steps) > 0.001 {
                *zoom_steps -= 1;
            }

            *handled = true;
        }

        if !keyboard_captured && matches!(event, Event::Text(text) if text == "~") {
            *centre = Vec3::zero();
        }

        if let Event::KeyPress {
            kind: k @ (Key::X | Key::Y | Key::Z),
            modifiers,
            handled,
            ..
        } = event
        {
            if *handled {
                continue;
            }

            let step = grid::interval(zoom_magnification(*zoom_steps));
            let offset = if modifiers.shift { -step } else { step };

            let coordinate = match k {
                Key::X => &mut centre.x,
                Key::Y => &mut centre.y,
                Key::Z => &mut centre.z,
                _ => unreachable!(),
            };

            *coordinate = ((*coordinate + offset) / step).round() * step;

            *handled = true;
        }
    }
}

/// Entry point?
///
/// # Errors
///
/// Window failed to be created.
///
/// # Panics
///
/// Error occurs during render loop.
pub fn main() -> Result<()> {
    let window = Window::new(WindowSettings {
        title: "Matrix Visualizer".to_string(),
        ..Default::default()
    })?;

    let ctx = window.gl();
    let mut gui = GUI::new(&ctx);
    let mut ui = ui::Ui::new(gui.context());

    // A "perspective" camera (as opposed to an orthographic one) is
    // often preferred as it's more natural to the eye. Though, for a
    // graphing tool, orthographic may be desired for its consistency.
    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(9.0, -12.0, 9.0),
        Vec3::zero(),
        Vec3::unit_z(),
        degrees(45.0),
        0.1,
        100.0,
    );

    // stored in graph coordinates
    let mut centre = Vec3::zero();
    let mut zoom_steps = 0;
    let magnification = zoom_magnification(zoom_steps);

    let mut grid_gm = Gm::new(
        grid::mesh(&ctx, magnification).expect("Failed to construct grid mesh!"),
        ColorMaterial::default(),
    );

    let mut grid_labels = Labels::new(&ctx, centre, magnification)?;
    grid_labels.face_camera(&camera);

    let origin_radius = 0.1;
    let mut origin_marker = Gm::new(
        Mesh::new(&ctx, &CpuMesh::sphere(16)),
        ColorMaterial {
            color: palette::AXES,
            ..Default::default()
        },
    );
    origin_marker.set_transformation(Mat4::from_scale(origin_radius));
    let mut origin_visible = true;

    window.render_loop(move |mut frame_input| {
        let graph_viewport = ui.update(&mut gui, &mut frame_input);
        camera.set_viewport(graph_viewport);

        let prev_zoom_steps = zoom_steps;
        let prev_centre = centre;
        let prev_camera_pos = camera.position();

        input_handler(
            &mut frame_input.events,
            &mut camera,
            &mut centre,
            &mut zoom_steps,
            gui.context().egui_wants_keyboard_input(),
            gui.context().egui_wants_pointer_input(),
        );

        let magnification = zoom_magnification(zoom_steps);
        let zoom_changed = zoom_steps != prev_zoom_steps;
        let graph_changed = zoom_changed || centre != prev_centre;
        let camera_pos = camera.position();

        let heading_changed = camera_pos.y.atan2(camera_pos.x).to_bits()
            != prev_camera_pos.y.atan2(prev_camera_pos.x).to_bits();

        if graph_changed || heading_changed {
            grid_labels.face_camera(&camera);
        }

        if graph_changed {
            grid_labels
                .rebuild(&ctx, centre, magnification)
                .expect("Failed to rebuild grid labels!");

            let origin_position = -centre * magnification;

            // keep marker size constant
            origin_marker.set_transformation(
                Mat4::from_translation(origin_position) * Mat4::from_scale(origin_radius),
            );

            origin_visible = [origin_position.x, origin_position.y, origin_position.z]
                .into_iter()
                .all(|coord| coord.abs() + origin_radius <= grid::GRID_SIZE);
        }

        if zoom_changed {
            grid_gm.geometry =
                grid::mesh(&ctx, magnification).expect("Failed to construct grid mesh!");
        }

        let (r, g, b) = palette::BACKGROUND;

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(r, g, b, 1.0, 1.0))
            .render_partially(
                graph_viewport.into(),
                &camera,
                iter::once(&grid_gm)
                    .chain(origin_visible.then_some(&origin_marker))
                    .chain(grid_labels.objects()),
                &[],
            )
            .write(|| gui.render())
            .expect("Failed to render UI!");

        FrameOutput::default()
    });

    Ok(())
}
