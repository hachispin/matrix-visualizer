//! Module responsible for handling and rendering the grid.
#![allow(unused, reason = "WIP")]

use anyhow::{Result, bail};
use three_d::{
    ColorMaterial,
    Context,
    CpuMesh,
    Gm,
    Indices,
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
#[derive(Debug)]
pub struct GridShape {
    /// Shape to render when no transformation is applied.
    original: ShapeType,
    /// Computed based off `transformations` on `original`.
    ///
    /// Must have the same number of points as `original`.
    transformed: ShapeType,
    /// Transformations to apply on `original`.
    transformations: Vec<Mat3>,
    /// The color of the shape.
    color: Srgba,
}

/// Enum representing possible shapes.
#[derive(Debug, Clone)]
pub enum ShapeType {
    Point(Vector3<f32>),
    Line(Vector3<f32>, Vector3<f32>),
    /// Connected in order of points. Last point connects to first.
    Polygon(Vec<Vector3<f32>>),
}

impl GridShape {
    /// Constructor.
    ///
    /// # Errors
    ///
    /// If the `shape` is a `Polygon` variant but has less than three points.
    pub fn new(shape: ShapeType, color: Srgba) -> Result<Self> {
        if let ShapeType::Polygon(points) = &shape
            && points.len() < 3
        {
            bail!("A polygon needs at least three points");
        }

        Ok(Self {
            original: shape.clone(),
            transformed: shape, // no transformations applied yet
            transformations: Vec::new(),
            color,
        })
    }
}

/// Represents a 3D grid.
///
/// The camera shouldn't be able to see outside the grid.
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
            shapes: Vec::new(),
            mesh: None,
            redraw_mesh: true,
        }
    }
}

impl PlottingGrid {
    pub fn new(centre: Vec3, magnification: f32, shapes: Vec<GridShape>) -> Self {
        Self {
            centre,
            magnification,
            shapes,
            ..Default::default()
        }
    }

    pub fn shapes(&self) -> &[GridShape] { &self.shapes }

    /// Convenience method for adding a shape.
    ///
    /// Equivalent to pushing to [`Self::mut_shapes`].
    pub fn push_shape(&mut self, shape: GridShape) {
        self.shapes.push(shape);
        self.redraw_mesh = true;
    }

    /// Returns a mutable references to the stored grid shapes.
    pub fn mut_shapes(&mut self) -> &mut [GridShape] {
        self.redraw_mesh = true;
        &mut self.shapes
    }

    /// Returns the mesh, redrawing if needed.
    ///
    /// This needs a context for the mesh to attach to.
    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::arithmetic_side_effects,
        reason = "Will fix later"
    )]
    pub fn mesh(&mut self, ctx: &Context) -> Option<&Mesh> {
        /// The grid extends out `GRID_SIZE` units from its origin in all six directions.
        const GRID_SIZE: i32 = 5;
        /// Width of quads that form the grid.
        const LINE_WIDTH: f32 = 0.05;
        /// For reserving capacity. 10.0 ** 2.
        const NUM_LINES: usize = 100;

        if !self.redraw_mesh {
            return self.const_mesh();
        }

        // Draws a basic grid.
        //
        // NOTE: Not updated by zoom functions yet!
        let mut positions = Vec::new();
        let mut indices = Vec::new();

        for i in -GRID_SIZE..=GRID_SIZE {
            let y = i as f32;

            let base = positions.len() as u32;
            positions.push(vec3(-GRID_SIZE as f32, y - LINE_WIDTH, 0.0));
            positions.push(vec3(GRID_SIZE as f32, y - LINE_WIDTH, 0.0));
            positions.push(vec3(GRID_SIZE as f32, y + LINE_WIDTH, 0.0));
            positions.push(vec3(-GRID_SIZE as f32, y + LINE_WIDTH, 0.0));

            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        for i in -GRID_SIZE..=GRID_SIZE {
            let x = i as f32;

            let base = positions.len() as u32;

            positions.push(vec3(x - LINE_WIDTH, -GRID_SIZE as f32, 0.0));
            positions.push(vec3(x + LINE_WIDTH, -GRID_SIZE as f32, 0.0));
            positions.push(vec3(x + LINE_WIDTH, GRID_SIZE as f32, 0.0));
            positions.push(vec3(x - LINE_WIDTH, GRID_SIZE as f32, 0.0));

            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        let positions_len = positions.len();

        let cpu_mesh = CpuMesh {
            positions: Positions::F32(positions),
            indices: Indices::U32(indices),
            colors: Some(vec![Srgba::RED; positions_len]),
            ..Default::default()
        };

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
