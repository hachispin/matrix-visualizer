//! For shapes.

#![allow(unused)]

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
