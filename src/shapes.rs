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

#[derive(Clone, Debug, PartialEq)]
pub struct LineSegment {
    pub id: usize,
    pub endpoints: (Vec3, Vec3),
    pub hidden: bool,
}

impl Default for LineSegment {
    fn default() -> Self {
        Self {
            id: 0,
            endpoints: (vec3(-1.0, -1.0, 0.0), vec3(1.0, 1.0, 0.0)),
            hidden: false,
        }
    }
}

impl LineSegment {
    pub fn color(&self) -> Srgba { palette::SHAPE_COLORS[self.id % palette::SHAPE_COLORS.len()] }
}

struct RenderedSegment {
    id: usize,
    object: Option<Gm<Mesh, ColorMaterial>>,
}

pub struct Segments {
    objects: Vec<RenderedSegment>,
    alpha: u8,
}

impl Default for Segments {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            alpha: 255,
        }
    }
}

impl Segments {
    pub fn translucent() -> Self {
        Self {
            alpha: 80,
            ..Self::default()
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        segments: &[LineSegment],
        transformation: Mat3,
        centre: Vec3,
        magnification: f32,
    ) -> anyhow::Result<()> {
        self.objects
            .retain(|object| segments.iter().any(|segment| segment.id == object.id));

        for segment in segments {
            let index = self
                .objects
                .iter()
                .position(|object| object.id == segment.id)
                .unwrap_or_else(|| {
                    self.objects.push(RenderedSegment {
                        id: segment.id,
                        object: None,
                    });

                    self.objects.len() - 1
                });

            let rendered = &mut self.objects[index];

            if segment.hidden {
                rendered.object = None;

                continue;
            }

            let start =
                utils::world_position(segment.endpoints.0, transformation, centre, magnification);
            let end =
                utils::world_position(segment.endpoints.1, transformation, centre, magnification);
            let mut positions = Vec::new();
            let mut indices = Vec::new();

            if let Some((start, end)) = utils::clip_edge(start, end) {
                utils::append_edge(&mut positions, &mut indices, start, end)?;
            }

            let mut color = segment.color();
            color.a = self.alpha;

            utils::update_mesh(ctx, &mut rendered.object, positions, indices, color);
        }

        Ok(())
    }

    pub fn objects(&self) -> impl Iterator<Item = &Gm<Mesh, ColorMaterial>> {
        self.objects
            .iter()
            .flat_map(|segment| segment.object.iter())
    }
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
