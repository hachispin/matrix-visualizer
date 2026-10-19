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
    Event,
    FrameOutput,
    Gm,
    Key,
    MouseButton,
    Object,
    Window,
    WindowSettings,
    prelude::*,
};

/// Returns a boolean indicating if the grid is to be redrawn.
///
/// Changes only occur through this so. Should work.
fn input_handler(
    events: &mut [Event],
    camera: &mut Camera,
    centre: &mut Vec3,
    magnification: &mut f32,
) -> bool {
    let mut dirty = false;

    // rotate the camera around the graph on drag
    for event in events {
        const DRAG_MULTIPLIER: f32 = 0.005;
        const ZOOM_FACTOR: f32 = 1.1;

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
            if delta.1 > 0.0 {
                *magnification *= ZOOM_FACTOR;
            } else if delta.1 < 0.0 {
                *magnification /= ZOOM_FACTOR;
            }

            *handled = true;
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

            let offset = if modifiers.shift { -1.0 } else { 1.0 };

            match k {
                Key::X => centre.x += offset,
                Key::Y => centre.y += offset,
                Key::Z => centre.z += offset,
                _ => unreachable!(),
            }

            dbg!(&centre, &magnification);

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

    // NOTE: The centre (or more like offset) is stored in grid squares!
    let mut centre = Vec3::zero();
    let mut magnification = 1.0;

    let mut grid_gm = Gm::new(
        grid::mesh(&ctx, magnification).expect("Failed to construct grid mesh!"),
        ColorMaterial::default(),
    );

    let mut grid_labels = labels::Labels::new(&ctx, centre, magnification)?;

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);

        let dirty = input_handler(
            &mut frame_input.events,
            &mut camera,
            &mut centre,
            &mut magnification,
        );

        magnification = magnification.clamp(0.001, 1000.0);

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
                std::iter::once(&grid_gm as &dyn Object).chain(grid_labels.objects()),
                &[],
            );

        FrameOutput::default()
    });

    Ok(())
}
