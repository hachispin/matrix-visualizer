//! Module responsible for handling and rendering the grid.
#![allow(unused, reason = "WIP")]

use anyhow::{Result, bail};
use three_d::{
    Context,
    CpuMesh,
    Indices,
    Mat3,
    Mat4,
    Mesh,
    Positions,
    Srgba,
    Vec3,
    Vector3,
    Zero,
    degrees,
    vec3,
};

use crate::{palette, utils};

/// Represents a 3D grid.
#[derive(Default)]
pub struct PlottingGrid {
    /// Initially [`None`].
    mesh: Option<Mesh>,
    // ↑  TODO: Should this initally be set to the mesh of a grid with default arguments?
}

impl PlottingGrid {
    pub fn new() -> Self { Self::default() }

    /// Draws -x → +x axis arrow. Transform accordingly for other axes.
    fn draw_axis_arrow() -> CpuMesh {
        // draws unit arrow starting at origin pointing to +x
        let mut arrow = CpuMesh::arrow(0.92, 0.2, 16);
        arrow.colors = Some(vec![palette::AXES; arrow.vertex_count()]);

        // translate back to negatives and scale up to grid size
        arrow
            .transform(
                Mat4::from_translation(vec3(-5.0, 0.0, 0.0))
                    * Mat4::from_nonuniform_scale(10.0, 0.12, 0.12),
            )
            .unwrap();

        arrow
    }

    /// Draws the mesh. Should only be called if needed as this draws on every call.
    pub fn mesh(&mut self, ctx: &Context, centre: Vec3, magnification: f32) -> Option<&Mesh> {
        // TODO: Respect `centre` and `magnification`.

        /// The cubic grid extends out `GRID_SIZE` world
        /// units from its origin in all six directions.
        const GRID_SIZE: i32 = 5;
        /// Width of quads that form the grid.
        const LINE_WIDTH: f32 = 0.05;

        const GRID_SIZE_F32: f32 = 5.0;

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
            (-GRID_SIZE_F32 / magnification).ceil() as i32,
            (GRID_SIZE_F32 / magnification).floor() as i32,
        );

        // this works out. to understand why take the case of 0.5x magnification.
        // the range doubles but each value `c` from the range is halved. so it
        // still ends up being inside the grid while just having more lines. we
        // basically just get a larger range and squish it to the same length.
        //
        // | magnification | range    | interpolated
        // | 0.5           | -10..=10 | -5, -4.5, …, 5
        // | 1             | -5..=5   | -5, -4, …, 5
        // | 2             | -2..=2   | -4, -2, …, 5
        //
        // this approach avoids the sort of unpredictable FPA precision loss (which would happen
        // in the trivial appraoch of just incrementing `c` by `spacing` until `GRID_SIZE`).
        for c in start..=end {
            let c = c as f32 * magnification;

            for (p1, p2) in [
                (vec3(0.0, c, 0.0), vec3(GRID_SIZE_F32, c, 0.0)),
                (vec3(0.0, c, 0.0), vec3(-GRID_SIZE_F32, c, 0.0)),
                (vec3(c, 0.0, 0.0), vec3(c, GRID_SIZE_F32, 0.0)),
                (vec3(c, 0.0, 0.0), vec3(c, -GRID_SIZE_F32, 0.0)),
            ] {
                utils::draw_2d_line(&mut positions, &mut indices, p1, p2, LINE_WIDTH);
            }
        }

        let num_positions = positions.len();

        let cpu_mesh = CpuMesh {
            positions: Positions::F32(positions),
            indices: Indices::U32(indices),
            colors: Some(vec![palette::MINOR_GRID; num_positions]),
            ..Default::default()
        };

        // axis meshes
        let x_axis = Self::draw_axis_arrow();

        let mut y_axis = Self::draw_axis_arrow();

        y_axis
            .transform(Mat4::from_angle_y(degrees(-90.0)))
            .unwrap();

        let mut z_axis = Self::draw_axis_arrow();

        z_axis.transform(Mat4::from_angle_z(degrees(90.0))).unwrap();

        let cpu_mesh = [cpu_mesh, x_axis, y_axis, z_axis]
            .into_iter()
            .fold(CpuMesh::default(), utils::merge_meshes);

        self.mesh = Some(Mesh::new(ctx, &cpu_mesh));

        self.mesh.as_ref()
    }

    /// Returns the currently stored mesh without redrawing.
    pub const fn const_mesh(&self) -> Option<&Mesh> { self.mesh.as_ref() }
}
