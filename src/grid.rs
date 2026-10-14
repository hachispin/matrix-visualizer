//! Module responsible for handling and rendering the grid.

use anyhow::Result;
use three_d::{Context, CpuMesh, Indices, Mesh, Positions, prelude::*};

use crate::{palette, utils};

/// The cubic grid extends out `GRID_SIZE` world units from its origin in all six directions.
const GRID_SIZE: f32 = 5.0;
/// Width of grid lines.
const LINE_WIDTH: f32 = 0.05;

/// Draws -x → +x axis arrow. Transform accordingly for other axes.
fn draw_axis_arrow() -> Result<CpuMesh> {
    // draws unit arrow starting at origin pointing to +x
    let mut arrow = CpuMesh::arrow(0.92, 0.2, 16);
    arrow.colors = Some(vec![palette::AXES; arrow.vertex_count()]);

    // translate back to negatives and scale up to grid size
    arrow.transform(
        Mat4::from_translation(vec3(-GRID_SIZE, 0.0, 0.0))
            * Mat4::from_nonuniform_scale(GRID_SIZE * 2.0, 0.12, 0.12),
    )?;

    Ok(arrow)
}

/// Draws the grid mesh. Should only be called if needed as this draws on every call.
pub fn mesh(ctx: &Context, _centre: Vec3, magnification: f32) -> Result<Mesh> {
    // TODO: Respect `centre`.

    // draw x-y grid
    //
    // axis lines should be drawn starting at x = 0 or y = 0 so the origin stays the origin
    // (since it's not guaranteed to fit an integer amount of gridlines at some zoom).
    //
    // also not planning on x/y axis stretching as a feature for now.

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    #[expect(
        clippy::cast_possible_truncation,
        reason = "iterating over a float range is annoying"
    )]
    let (start, end) = (
        (-GRID_SIZE / magnification).ceil() as i32,
        (GRID_SIZE / magnification).floor() as i32,
    );

    // this works out. to understand why take the case of 0.5x magnification.
    // the range doubles but each value `c` from the range is halved. so it
    // still ends up being inside the grid while just having more lines. we
    // basically just get a larger range and squish it to the same length.
    //
    // | magnification | range    | interpolated
    // | 0.5           | -10..=10 | -5, -4.5, …, 5
    // | 1             | -5..=5   | -5, -4, …, 5
    // | 2             | -2..=2   | -4, -2, …, 4
    //
    // this approach avoids the sort of accumulated FPA precision loss (which would happen
    // in the trivial appraoch of just incrementing `c` by `spacing` until `GRID_SIZE`).

    for c in start..=end {
        #[expect(
            clippy::cast_precision_loss,
            reason = "magnification is clamped in main"
        )]
        let c = c as f32 * magnification;

        // to not overlap with axes
        if c == 0.0 {
            continue;
        }

        for (p1, p2) in [
            (vec3(0.0, c, 0.0), vec3(GRID_SIZE, c, 0.0)),
            (vec3(0.0, c, 0.0), vec3(-GRID_SIZE, c, 0.0)),
            (vec3(c, 0.0, 0.0), vec3(c, GRID_SIZE, 0.0)),
            (vec3(c, 0.0, 0.0), vec3(c, -GRID_SIZE, 0.0)),
        ] {
            utils::draw_2d_line(&mut positions, &mut indices, p1, p2, LINE_WIDTH);
        }
    }

    let num_positions = positions.len();

    let cpu_mesh = CpuMesh {
        positions: Positions::F32(positions),
        indices: Indices::U32(indices),
        colors: Some(vec![palette::GRID; num_positions]),
        ..Default::default()
    };

    // axis meshes
    let x_axis = draw_axis_arrow()?;
    let mut y_axis = x_axis.clone();
    let mut z_axis = x_axis.clone();

    y_axis.transform(Mat4::from_angle_z(degrees(90.0)))?;
    z_axis.transform(Mat4::from_angle_y(degrees(-90.0)))?;

    let cpu_mesh = [cpu_mesh, x_axis, z_axis, y_axis]
        .into_iter()
        .fold(CpuMesh::default(), utils::merge_meshes);

    Ok(Mesh::new(ctx, &cpu_mesh))
}
