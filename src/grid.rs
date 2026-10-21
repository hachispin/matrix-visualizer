//! Module responsible for handling and rendering the grid.

use anyhow::Result;
use three_d::{Context, CpuMesh, Indices, Mesh, Positions, prelude::*};

use crate::{palette, utils};

/// The cubic grid extends out `GRID_SIZE` world units from its origin in all six directions.
pub(crate) const GRID_SIZE: f32 = 5.0;

/// Width of grid lines.
const LINE_WIDTH: f32 = 0.05;

/// Whether a world-space point and its marker radius fit inside the grid bounds.
pub fn contains(position: Vec3, radius: f32) -> bool {
    [position.x, position.y, position.z]
        .into_iter()
        .all(|coordinate| coordinate.abs() + radius <= GRID_SIZE)
}

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

/// Largest 1–2–5 graph interval that fits within one world unit at this zoom.
/// This keeps gridline spacing between 0.4 and 1 world units.
pub(crate) fn interval(magnification: f32) -> f32 {
    let target = f64::from(magnification).recip();
    let decade = 10.0_f64.powf(target.log10().floor());
    let fraction = target / decade;

    let factor = if fraction >= 5.0 {
        5.0
    } else if fraction >= 2.0 {
        2.0
    } else {
        1.0
    };

    (factor * decade) as f32
}

/// Whether the zoom and the graph coordinates throughout the visible grid remain finite.
pub(crate) fn view_is_finite(centre: Vec3, magnification: f32) -> bool {
    if !magnification.is_finite() || magnification <= 0.0 {
        return false;
    }

    let extent = GRID_SIZE / magnification;

    [centre.x, centre.y, centre.z]
        .into_iter()
        .all(|coordinate| (coordinate - extent).is_finite() && (coordinate + extent).is_finite())
}

/// Draws the grid mesh. Should only be called if needed as this draws on every call.
///
/// This doesn't need to know what the centre is. The grid stays the same regardless. Think of it
/// like this: the grid is not moved through the graph - the graph is being moved through the grid.
pub fn mesh(ctx: &Context, magnification: f32) -> Result<Mesh> {
    let spacing = interval(magnification) * magnification;

    // draw x-y grid
    //
    // axis lines should be drawn starting at x = 0 or y = 0 so the origin stays the origin
    // (since it's not guaranteed to fit an integer amount of gridlines at some zoom).
    //
    // also not planning on x/y axis stretching as a feature for now.

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    let (start, end) = (
        (-GRID_SIZE / spacing).ceil() as i32,
        (GRID_SIZE / spacing).floor() as i32,
    );

    // Multiply integer tick indices by the spacing to avoid accumulating floating-point error.

    for c in start..=end {
        let c = c as f32 * spacing;

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
            utils::draw_2d_line(&mut positions, &mut indices, p1, p2, LINE_WIDTH)?;
        }
    }

    let num_positions = positions.len();

    let gridlines = CpuMesh {
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

    let cpu_mesh = [gridlines, x_axis, z_axis, y_axis]
        .into_iter()
        .try_fold(CpuMesh::default(), utils::merge_meshes)?;

    Ok(Mesh::new(ctx, &cpu_mesh))
}
