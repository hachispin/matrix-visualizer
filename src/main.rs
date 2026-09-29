#![warn(clippy::pedantic)]
#![allow(clippy::cast_precision_loss)]

use three_d::{
    Camera,
    ClearState,
    ColorMaterial,
    FrameOutput,
    Gm,
    Window,
    WindowSettings,
    degrees,
    vec3,
};

use crate::grid::PlottingGrid;

mod grid;
pub mod palette;

/// Rotates a rectangle.
///
/// # Panics
///
/// Window failed to be created.
pub fn main() {
    let window = Window::new(WindowSettings {
        title: "Rectangle".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();

    let ctx = window.gl();
    let mut grid = PlottingGrid::default();

    // A "perspective" camera (as opposed to an orthographic one) is
    // often preferred as it's more natural to the eye. Though, for a
    // graphing tool, orthographic may be desired for its consistency.
    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 0.0, 5.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        10.0,
    );

    window.render_loop(move |frame_input| {
        camera.set_viewport(frame_input.viewport);
        let gm = Gm::new(grid.mesh(&ctx).unwrap(), ColorMaterial::default());

        let bg = palette::BACKGROUND;

        // turn to normalized
        let (r, g, b) = (
            f32::from(bg.r) / 255.0,
            f32::from(bg.g) / 255.0,
            f32::from(bg.b) / 255.0,
        );

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(r, g, b, 1.0, 1.0))
            .render(&camera, &gm, &[]);

        FrameOutput::default()
    });
}
