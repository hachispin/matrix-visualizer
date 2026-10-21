//! For shapes.

use three_d::{
    ColorMaterial,
    Context,
    CpuMaterial,
    CpuMesh,
    Gm,
    Mat3,
    Mat4,
    Mesh,
    Srgba,
    Vec3,
    vec3,
};

use crate::{grid, palette, utils};

/// An editable point in graph coordinates. IDs keep its UI and GPU object stable.
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub id: usize,
    pub position: Vec3,
    pub hidden: bool,
}

impl Default for Point {
    fn default() -> Self {
        Self {
            id: 0,
            position: vec3(1.0, 1.0, 1.0),
            hidden: false,
        }
    }
}

impl Point {
    pub fn color(&self) -> Srgba { palette::SHAPE_COLORS[self.id % palette::SHAPE_COLORS.len()] }
}

struct RenderedPoint {
    id: usize,
    object: Gm<Mesh, ColorMaterial>,
    hidden: bool,
}

pub struct Points {
    mesh: CpuMesh,
    objects: Vec<RenderedPoint>,
    alpha: u8,
}

impl Default for Points {
    fn default() -> Self {
        Self {
            mesh: CpuMesh::sphere(16),
            objects: Vec::new(),
            alpha: 255,
        }
    }
}

impl Points {
    pub fn translucent() -> Self {
        Self {
            alpha: 80,
            ..Self::default()
        }
    }

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
                    let material = if self.alpha < 255 {
                        ColorMaterial::new_transparent(ctx, &CpuMaterial::default())
                    } else {
                        ColorMaterial::default()
                    };

                    self.objects.push(RenderedPoint {
                        id: point.id,
                        object: Gm::new(Mesh::new(ctx, &self.mesh), material),
                        hidden: true,
                    });

                    self.objects.len() - 1
                });

            let rendered = &mut self.objects[index];
            let position =
                utils::graph_to_world(transformation * point.position, centre, magnification);

            rendered.hidden = point.hidden || !grid::contains(position, RADIUS);

            if !rendered.hidden {
                rendered.object.material.color = point.color();
                rendered.object.material.color.a = self.alpha;
                rendered.object.set_transformation(
                    Mat4::from_translation(position) * Mat4::from_scale(RADIUS),
                );
            }
        }
    }

    pub fn objects(&self) -> impl Iterator<Item = &Gm<Mesh, ColorMaterial>> {
        self.objects
            .iter()
            .filter(|point| !point.hidden)
            .map(|point| &point.object)
    }
}
