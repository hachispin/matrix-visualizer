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
pub struct PlottingGrid {
    /// Initially (0, 0, 0).
    centre: Vec3,
    // At a magnification of 1, a plotting unit
    // should be the same size as a world unit.
    // NOTE: think about whether this should be constrained (having a max/min value)
    magnification: f32,
    /// The mesh. Will be updated when needed.
    ///
    /// Initially, this is set to `None`.
    mesh: Option<Mesh>,
    /// Flag set for whenever the mesh needs to be redrawn.
    redraw_mesh: bool,
}

impl Default for PlottingGrid {
    fn default() -> Self {
        Self {
            centre: Vec3::zero(),
            magnification: 1.0,
            mesh: None,
            redraw_mesh: true,
        }
    }
}

impl PlottingGrid {
    pub fn new(centre: Vec3, magnification: f32) -> Self {
        Self {
            centre,
            magnification,
            ..Default::default()
        }
    }

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

    /// Returns the mesh, redrawing if needed.
    ///
    /// This needs a context for the mesh to attach to.
    pub fn mesh(&mut self, ctx: &Context) -> Option<&Mesh> {
        /// The cubic grid extends out `GRID_SIZE` world
        /// units from its origin in all six directions.
        const GRID_SIZE: i32 = 5;
        /// Width of quads that form the grid.
        const LINE_WIDTH: f32 = 0.05;

        const GRID_SIZE_F32: f32 = 5.0;

        if !self.redraw_mesh {
            return self.const_mesh();
        }

        // draw x-y grid
        //
        // axis lines should be drawn starting at x = 0 or y = 0 so the origin stays the origin
        // (since it's not guaranteed to fit an integer amount of gridlines at some zoom).
        //
        // also not planning on x/y axis stretching as a feature for now.

        let mut positions = Vec::new();
        let mut indices = Vec::new();

        for c in -GRID_SIZE..=GRID_SIZE {
            let c = c as f32;

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
        self.redraw_mesh = false;

        self.mesh.as_ref()
    }

    /// Returns the mesh without redrawing.
    ///
    /// In other words, a simple getter for the mesh field.
    pub const fn const_mesh(&self) -> Option<&Mesh> { self.mesh.as_ref() }

    pub fn zoom_in(&mut self) {
        self.magnification *= 2.0;
        self.redraw_mesh = true;
        todo!()
    }

    pub fn zoom_out(&mut self) {
        self.magnification /= 2.0;
        self.redraw_mesh = true;
        todo!()
    }
}
