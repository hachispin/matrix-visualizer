//! For shapes.

use three_d::{ColorMaterial, Context, CpuMesh, Gm, Mat3, Mat4, Mesh, Srgba, Vec3};

use crate::{grid, utils};

/// An editable point in graph coordinates. IDs keep its UI and GPU object stable.
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub id: usize,
    pub position: Vec3,
    pub color: Srgba,
    pub visible: bool,
}

struct RenderedPoint {
    id: usize,
    object: Gm<Mesh, ColorMaterial>,
    visible: bool,
}

pub struct Points {
    mesh: CpuMesh,
    objects: Vec<RenderedPoint>,
}

impl Default for Points {
    fn default() -> Self {
        Self {
            mesh: CpuMesh::sphere(16),
            objects: Vec::new(),
        }
    }
}

impl Points {
    /// Update explicitly when point data or its graph-to-world mapping changes.
    pub fn update(
        &mut self,
        ctx: &Context,
        points: &[Point],
        transformation: Mat3,
        centre: Vec3,
        magnification: f32,
    ) {
        const RADIUS: f32 = 0.12;

        self.objects
            .retain(|object| points.iter().any(|point| point.id == object.id));

        for point in points {
            let index = self
                .objects
                .iter()
                .position(|object| object.id == point.id)
                .unwrap_or_else(|| {
                    self.objects.push(RenderedPoint {
                        id: point.id,
                        object: Gm::new(Mesh::new(ctx, &self.mesh), ColorMaterial::default()),
                        visible: false,
                    });

                    self.objects.len() - 1
                });

            let rendered = &mut self.objects[index];
            let position =
                utils::graph_to_world(transformation * point.position, centre, magnification);

            rendered.visible = point.visible && grid::contains(position, RADIUS);

            if rendered.visible {
                rendered.object.material.color = point.color;
                rendered.object.set_transformation(
                    Mat4::from_translation(position) * Mat4::from_scale(RADIUS),
                );
            }
        }
    }

    pub fn objects(&self) -> impl Iterator<Item = &Gm<Mesh, ColorMaterial>> {
        self.objects
            .iter()
            .filter(|point| point.visible)
            .map(|point| &point.object)
    }
}
