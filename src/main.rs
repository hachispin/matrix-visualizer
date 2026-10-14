#![warn(clippy::pedantic)]
#![allow(clippy::cast_precision_loss)]

mod grid;
mod palette;
mod shapes;
mod utils;

use three_d::{
    Camera,
    ClearState,
    ColorMaterial,
    Event,
    FrameOutput,
    Gm,
    MouseButton,
    Vec3,
    Window,
    WindowSettings,
    Zero,
    degrees,
    vec3,
};

/// Rotates a rectangle.
///
/// # Panics
///
/// Window failed to be created.
pub fn main() {
    let window = Window::new(WindowSettings {
        title: "Matrix Visualizer".to_string(),
        ..Default::default()
    })
    .unwrap();

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

    // TODO: Attach these to certain inputs.
    let centre = Vec3::zero();
    let mut magnification = 1.0;

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);

        // rotate the camera around the graph on drag
        for event in &mut frame_input.events {
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
            }

            if let Event::MouseWheel { delta, handled, .. } = event {
                if *handled {
                    continue;
                }

                // vertical scroll amount (positive = up)
                if delta.1 > 0.0 {
                    magnification *= ZOOM_FACTOR;
                } else if delta.1 < 0.0 {
                    magnification /= ZOOM_FACTOR;
                }

                *handled = true;
            }
        }

        let gm = Gm::new(
            grid::mesh(&ctx, centre, magnification),
            ColorMaterial::default(),
        );

        let (r, g, b) = palette::BACKGROUND;

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(r, g, b, 1.0, 1.0))
            .render(&camera, [&gm], &[]);

        FrameOutput::default()
    });
}
