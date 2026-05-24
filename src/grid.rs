//! Module responsible for handling and rendering the grid.
#![allow(unused, reason = "WIP")]

use std::ops::Range;

use three_d::{
    ColorMaterial,
    Context,
    CpuMesh,
    Gm,
    Mat3,
    Mesh,
    Positions,
    Srgba,
    Vec3,
    Vector3,
    Zero,
    vec3,
};

/// A shape rendered on the grid.
///
/// This can represent a point, line and polygon.
#[derive(Debug)]
struct GridShape {
    /// Points to render when no transformation is applied.
    ///
    /// If there is more than one point, then the points
    /// are connected in the order that they're declared.
    original: Vec<Vec3>,
    /// Computed based off `transformations` on `original_points`.
    ///
    /// Must be the same length as `original_points`.
    transformed: Vec<Vec3>,
    /// Transformations to apply on `original_points`.
    transformations: Vec<Mat3>,
    /// The color of the point and the connecting lines, if present.
    color: Srgba,
    /// Whether the last point should connect with the first.
    ///
    /// Has no impact if only two or less points are defined.
    closed: bool,
}

impl GridShape {
    pub const fn polygon(points: Vec<Vec3>, color: Srgba) -> Self {
        Self {
            original: points,
            transformed: Vec::new(),
            transformations: Vec::new(),
            color,
            closed: true,
        }
    }

    pub fn line(p1: Vec3, p2: Vec3, color: Srgba) -> Self {
        Self {
            original: vec![p1, p2],
            transformed: Vec::new(),
            transformations: Vec::new(),
            color,
            closed: false,
        }
    }

    pub fn point(point: Vec3, color: Srgba) -> Self {
        Self {
            original: vec![point],
            transformed: Vec::new(),
            transformations: Vec::new(),
            color,
            closed: false,
        }
    }
}

/// Represents a 3D grid.
///
/// Generally should be declared as mutable.
pub struct PlottingGrid {
    /// Initially (0, 0, 0).
    centre: Vec3,
    // At a magnification of 1, a plotting unit
    // should be the same size as a world unit.
    // NOTE: think about whether this should be constrained (having a max/min value)
    magnification: f32,
    /// Shapes to render.
    shapes: Vec<GridShape>,
    /// The mesh. Will be updated when needed.
    mesh: CpuMesh,
    /// Flag set for whenever the mesh needs to be redrawn.
    redraw_mesh: bool,
}

impl Default for PlottingGrid {
    fn default() -> Self {
        Self {
            centre: Vec3::zero(),
            magnification: 1.0,
            shapes: Vec::new(),
            mesh: CpuMesh::default(),
            redraw_mesh: false,
        }
    }
}

impl PlottingGrid {
    fn new(centre: Vec3, magnification: f32, shapes: Vec<GridShape>) -> Self {
        Self {
            centre,
            magnification,
            shapes,
            ..Default::default()
        }
    }

    fn shapes(&self) -> &[GridShape] { &self.shapes }

    /// Convenience method for adding a shape.
    ///
    /// Equivalent to pushing to [`Self::mut_shapes`].
    fn push_shape(&mut self, shape: GridShape) {
        self.shapes.push(shape);
        self.redraw_mesh = true;
    }

    /// Returns a mutable references to the stored grid shapes.
    fn mut_shapes(&mut self) -> &mut [GridShape] {
        self.redraw_mesh = true;
        &mut self.shapes
    }

    /// Recomputes the stored mesh if it needs changes.
    ///
    /// One example for when the mesh would be redrawn is if
    /// a new shape is added (through [`Self::add_shape`]).
    ///
    /// This resets the `redraw_mesh` flag to false afterwards.
    fn redraw_if_needed(&mut self) {
        if self.redraw_mesh {
            /* ... */
            self.redraw_mesh = false;
        }

        todo!()
    }

    /// Returns the mesh.
    ///
    /// NOTE: Make sure to call [`Self::redraw_if_needed`] if the mesh has been modified.
    const fn mesh(&self) -> &CpuMesh { &self.mesh }

    fn zoom_in(&mut self) {
        self.magnification *= 2.0;
        self.redraw_mesh = true;
        todo!()
    }

    fn zoom_out(&mut self) {
        self.magnification /= 2.0;
        self.redraw_mesh = true;
        todo!()
    }
}
