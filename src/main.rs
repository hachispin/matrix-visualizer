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

mod grid;
mod labels;
mod palette;
mod shapes;
mod utils;

use anyhow::Result;
use three_d::{
    Camera,
    ClearState,
    ColorMaterial,
    CpuMesh,
    Event,
    FrameOutput,
    Gm,
    Key,
    Mesh,
    MouseButton,
    Object,
    Window,
    WindowSettings,
    prelude::*,
};

/// Recompute from integer wheel steps so zooming back never accumulates rounding error.
fn zoom_magnification(steps: i32) -> f32 { 1.1_f32.powi(steps).clamp(0.001, 1000.0) }

/// Returns a boolean indicating if the grid is to be redrawn.
///
/// Changes only occur through this so. Should work.
fn input_handler(
    events: &mut [Event],
    camera: &mut Camera,
    centre: &mut Vec3,
    zoom_steps: &mut i32,
) -> bool {
    let mut dirty = false;

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
            if *handled {
                continue;
            }

            camera.rotate_around_with_fixed_up(
                Vec3::zero(),
                DRAG_MULTIPLIER * delta.0,
                DRAG_MULTIPLIER * delta.1,
            );

            // prevent other event handlers from using this event again
            *handled = true;
            dirty = true;
        }

        if let Event::MouseWheel { delta, handled, .. } = event {
            if *handled {
                continue;
            }

            // vertical scroll amount (positive = up)
            // Bound the step count too, so scrolling at a limit does not build up overscroll.
            if delta.1 > 0.0 && zoom_magnification(*zoom_steps) < 1000.0 {
                *zoom_steps += 1;
            } else if delta.1 < 0.0 && zoom_magnification(*zoom_steps) > 0.001 {
                *zoom_steps -= 1;
            }

            *handled = true;
            dirty = true;
        }

        // Text input also catches tilde on layouts where it is not on the grave key.
        if matches!(event, Event::Text(text) if text == "~") {
            *centre = Vec3::zero();
            dirty = true;
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

            dbg!(&centre, zoom_magnification(*zoom_steps));

            *handled = true;
            dirty = true;
        }
    }

    dirty
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

    // The centre is stored in graph coordinates.
    let mut centre = Vec3::zero();
    let mut zoom_steps = 0;
    let magnification = zoom_magnification(zoom_steps);

    let mut grid_gm = Gm::new(
        grid::mesh(&ctx, magnification).expect("Failed to construct grid mesh!"),
        ColorMaterial::default(),
    );

    let mut grid_labels = labels::Labels::new(&ctx, centre, magnification)?;

    let origin_radius = 0.1;
    let mut origin_marker = Gm::new(
        Mesh::new(&ctx, &CpuMesh::sphere(16)),
        ColorMaterial {
            color: palette::AXES,
            ..Default::default()
        },
    );

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);

        let dirty = input_handler(
            &mut frame_input.events,
            &mut camera,
            &mut centre,
            &mut zoom_steps,
        );

        let magnification = zoom_magnification(zoom_steps);

        // Follow the graph origin, keeping the marker's world size constant at every zoom.
        let origin_position = -centre * magnification;
        origin_marker.set_transformation(
            Mat4::from_translation(origin_position) * Mat4::from_scale(origin_radius),
        );
        let origin_visible = [origin_position.x, origin_position.y, origin_position.z]
            .into_iter()
            .all(|coordinate| coordinate.abs() + origin_radius <= grid::GRID_SIZE);

        grid_labels
            .update(&ctx, &camera, centre, magnification)
            .expect("Failed to update grid labels!");

        if dirty {
            grid_gm = Gm::new(
                grid::mesh(&ctx, magnification).expect("Failed to construct grid mesh!"),
                ColorMaterial::default(),
            );
        }

        let (r, g, b) = palette::BACKGROUND;

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(r, g, b, 1.0, 1.0))
            .render(
                &camera,
                std::iter::once(&grid_gm as &dyn Object)
                    .chain(origin_visible.then_some(&origin_marker as &dyn Object))
                    .chain(grid_labels.objects()),
                &[],
            );

        FrameOutput::default()
    });

    Ok(())
}
