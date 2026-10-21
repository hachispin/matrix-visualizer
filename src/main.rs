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

mod graph;
mod grid;
mod labels;
mod palette;
mod polygons;
mod shapes;
mod ui;
mod utils;

#[cfg(target_arch = "wasm32")]
mod web_input;

use anyhow::Result;
use three_d::{
    Camera,
    ClearState,
    Event,
    FrameOutput,
    GUI,
    Key,
    MouseButton,
    Window,
    WindowSettings,
    prelude::*,
};

use crate::graph::Graph;

// this prevents drift (from repeatedly multiplying and dividing by 1.1)
fn zoom_magnification(steps: i32) -> f32 { 1.1_f64.powi(steps) as f32 }

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

            // Stop only when the next zoom would exceed the numeric range of the view.
            let direction = if delta.1 > 0.0 {
                1
            } else if delta.1 < 0.0 {
                -1
            } else {
                0
            };

            if let Some(candidate) = zoom_steps.checked_add(direction)
                && grid::view_is_finite(*centre, zoom_magnification(candidate))
            {
                *zoom_steps = candidate;
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

            let magnification = zoom_magnification(*zoom_steps);
            let step = f64::from(grid::interval(magnification));
            let offset = if modifiers.shift { -step } else { step };

            let mut candidate = *centre;

            let coordinate = match k {
                Key::X => &mut candidate.x,
                Key::Y => &mut candidate.y,
                Key::Z => &mut candidate.z,
                _ => unreachable!(),
            };

            *coordinate = (((f64::from(*coordinate) + offset) / step).round() * step) as f32;

            if grid::view_is_finite(candidate, magnification) {
                *centre = candidate;
            }

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
    let mut graph = Graph::new(&ctx, &camera)?;

    window.render_loop(move |mut frame_input| {
        let graph_viewport = ui.update(&mut gui, &mut frame_input);
        camera.set_viewport(graph_viewport);

        input_handler(
            &mut frame_input.events,
            &mut camera,
            &mut centre,
            &mut zoom_steps,
            gui.context().egui_wants_keyboard_input(),
            gui.context().egui_wants_pointer_input(),
        );

        graph
            .update(&ctx, &camera, centre, zoom_magnification(zoom_steps), &ui)
            .expect("Failed to update graph!");

        let (r, g, b) = palette::BACKGROUND;

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(r, g, b, 1.0, 1.0))
            .render_partially(graph_viewport.into(), &camera, graph.objects(), &[])
            .write(|| gui.render())
            .expect("Failed to render UI!");

        FrameOutput::default()
    });

    Ok(())
}
