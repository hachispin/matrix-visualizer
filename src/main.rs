use three_d::{
    Camera,
    ClearState,
    ColorMaterial,
    FrameOutput,
    Gm,
    Mesh,
    Window,
    WindowSettings,
    degrees,
    vec3,
};

use crate::grid::PlottingGrid;

mod grid;

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
        let gm = Gm::new(Mesh::new(&ctx, grid.mesh()), ColorMaterial::default());

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(&camera, &gm, &[]);

        FrameOutput::default()
    });
}
