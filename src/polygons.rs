//! Editable polygons, sequential outlines, and triangulated translucent interiors.

use std::collections::BTreeSet;

use anyhow::Result;
use three_d::{ColorMaterial, Context, Gm, Mat3, Mesh, Srgba, Vec3, Vector3, Zero};

use crate::{grid, palette, utils};

#[derive(Clone, Debug, PartialEq)]
pub struct Vertex {
    pub id: usize,
    pub position: Vec3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Polygon {
    pub id: usize,
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Vec<usize>>,
    pub hidden: bool,
}

impl Default for Polygon {
    fn default() -> Self {
        Self {
            id: 0,
            vertices: [Vec3::zero(); 3]
                .into_iter()
                .enumerate()
                .map(|(id, position)| Vertex { id, position })
                .collect(),
            faces: Vec::new(),
            hidden: false,
        }
    }
}

impl Polygon {
    pub const fn color(&self) -> Srgba {
        palette::SHAPE_COLORS[self.id % palette::SHAPE_COLORS.len()]
    }

    pub fn make_unit_cube(&mut self) {
        self.vertices = [
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new(0.5, -0.5, -0.5),
            Vec3::new(0.5, 0.5, -0.5),
            Vec3::new(-0.5, 0.5, -0.5),
            Vec3::new(-0.5, -0.5, 0.5),
            Vec3::new(0.5, -0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(-0.5, 0.5, 0.5),
        ]
        .into_iter()
        .enumerate()
        .map(|(id, position)| Vertex { id, position })
        .collect();
        self.faces = [
            [0, 3, 2, 1],
            [4, 5, 6, 7],
            [0, 1, 5, 4],
            [1, 2, 6, 5],
            [2, 3, 7, 6],
            [3, 0, 4, 7],
        ]
        .into_iter()
        .map(Vec::from)
        .collect();
    }

    fn triangles(&self) -> Vec<[usize; 3]> {
        if self.faces.is_empty() {
            return triangulate(&self.vertices);
        }

        self.faces
            .iter()
            .flat_map(|face| {
                let vertices: Vec<_> = face
                    .iter()
                    .map(|&index| self.vertices[index].clone())
                    .collect();

                triangulate(&vertices)
                    .into_iter()
                    .map(|triangle| triangle.map(|index| face[index]))
            })
            .collect()
    }

    fn edges(&self) -> Vec<(usize, usize)> {
        if self.faces.is_empty() {
            return (0..self.vertices.len())
                .map(|index| (index, (index + 1) % self.vertices.len()))
                .collect();
        }

        self.faces
            .iter()
            .flat_map(|face| {
                face.iter()
                    .zip(face.iter().cycle().skip(1))
                    .map(|(&start, &end)| (start.min(end), start.max(end)))
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

struct RenderedPolygon {
    id: usize,
    fill: Option<Gm<Mesh, ColorMaterial>>,
    outline: Option<Gm<Mesh, ColorMaterial>>,
}

pub struct Polygons {
    objects: Vec<RenderedPolygon>,
    alpha: u8,
}

impl Default for Polygons {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            alpha: 255,
        }
    }
}

impl Polygons {
    pub fn translucent() -> Self {
        Self {
            alpha: 80,
            ..Self::default()
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        polygons: &[Polygon],
        transformation: Mat3,
        centre: Vec3,
        magnification: f32,
    ) -> Result<()> {
        self.objects
            .retain(|object| polygons.iter().any(|polygon| polygon.id == object.id));

        for polygon in polygons {
            let index = self
                .objects
                .iter()
                .position(|object| object.id == polygon.id)
                .unwrap_or_else(|| {
                    self.objects.push(RenderedPolygon {
                        id: polygon.id,
                        fill: None,
                        outline: None,
                    });

                    self.objects.len() - 1
                });

            let rendered = &mut self.objects[index];

            if polygon.hidden {
                rendered.fill = None;
                rendered.outline = None;

                continue;
            }

            let positions: Vec<_> = polygon
                .vertices
                .iter()
                .map(|vertex| {
                    utils::world_position(vertex.position, transformation, centre, magnification)
                })
                .collect();

            let mut fill_positions = Vec::new();
            let mut fill_indices = Vec::new();

            for triangle in polygon.triangles() {
                let clipped = clip_triangle(triangle.map(|index| positions[index]));

                if clipped.len() < 3 {
                    continue;
                }

                let base = u32::try_from(fill_positions.len())?;

                fill_positions.extend(
                    clipped
                        .iter()
                        .map(|position| position.map(|value| value as f32)),
                );

                for index in 1..clipped.len() - 1 {
                    let index = u32::try_from(index)?;

                    fill_indices.extend([base, base + index, base + index + 1]);
                }
            }

            let mut outline_positions = Vec::new();
            let mut outline_indices = Vec::new();

            for (start, end) in polygon.edges() {
                if let Some((start, end)) = utils::clip_edge(positions[start], positions[end]) {
                    utils::append_edge(&mut outline_positions, &mut outline_indices, start, end)?;
                }
            }

            let mut color = polygon.color();
            color.a = self.alpha / 4;

            utils::update_mesh(ctx, &mut rendered.fill, fill_positions, fill_indices, color);

            color.a = self.alpha;

            utils::update_mesh(
                ctx,
                &mut rendered.outline,
                outline_positions,
                outline_indices,
                color,
            );
        }

        Ok(())
    }

    pub fn objects(&self) -> impl Iterator<Item = &Gm<Mesh, ColorMaterial>> {
        self.objects
            .iter()
            .flat_map(|polygon| polygon.fill.iter().chain(polygon.outline.iter()))
    }
}

fn clip_triangle(triangle: [Vector3<f64>; 3]) -> Vec<Vector3<f64>> {
    let mut vertices = triangle.to_vec();

    for axis in 0..3 {
        for sign in [-1.0, 1.0] {
            let input = std::mem::take(&mut vertices);
            let Some(&last) = input.last() else {
                return vertices;
            };
            let mut start = last;

            for end in input {
                let start_distance = f64::mul_add(sign, -start[axis], f64::from(grid::GRID_SIZE));
                let end_distance = f64::mul_add(sign, -end[axis], f64::from(grid::GRID_SIZE));

                if (start_distance >= 0.0) != (end_distance >= 0.0) {
                    let amount = start_distance / (start_distance - end_distance);
                    let mut intersection = start + (end - start) * amount;
                    intersection[axis] = sign * f64::from(grid::GRID_SIZE);
                    vertices.push(intersection);
                }

                if end_distance >= 0.0 {
                    vertices.push(end);
                }

                start = end;
            }
        }
    }

    vertices
}

/// Ear clipping preserves the interior of concave polygons and either winding order.
fn triangulate(vertices: &[Vertex]) -> Vec<[usize; 3]> {
    if vertices.len() < 3 {
        return Vec::new();
    }

    let origin = vertices[0].position.map(f64::from);
    let mut positions: Vec<_> = vertices
        .iter()
        .map(|vertex| vertex.position.map(f64::from) - origin)
        .collect();
    let scale = positions
        .iter()
        .flat_map(|position| [position.x.abs(), position.y.abs(), position.z.abs()])
        .fold(0.0_f64, f64::max);

    if scale == 0.0 {
        return Vec::new();
    }

    for position in &mut positions {
        *position /= scale;
    }

    let normal = positions
        .iter()
        .zip(positions.iter().cycle().skip(1))
        .fold(Vector3::new(0.0, 0.0, 0.0), |normal, (start, end)| {
            normal + start.cross(*end)
        });
    let drop_axis = (0..3)
        .max_by(|&a, &b| normal[a].abs().total_cmp(&normal[b].abs()))
        .unwrap_or(2);
    let projected: Vec<_> = positions
        .iter()
        .map(|position| match drop_axis {
            0 => [position.y, position.z],
            1 => [position.x, position.z],
            _ => [position.x, position.y],
        })
        .collect();
    let area: f64 = projected
        .iter()
        .zip(projected.iter().cycle().skip(1))
        .map(|(a, b)| f64::mul_add(b[0], -a[1], a[0] * b[1]))
        .sum();

    if area.abs() < 1.0e-12 {
        return Vec::new();
    }

    let winding = area.signum();
    let mut remaining: Vec<_> = (0..vertices.len()).collect();
    let mut triangles = Vec::new();

    'triangulate: while remaining.len() > 3 {
        for index in 0..remaining.len() {
            let previous = remaining[(index + remaining.len() - 1) % remaining.len()];
            let current = remaining[index];
            let next = remaining[(index + 1) % remaining.len()];
            let a = projected[previous];
            let b = projected[current];
            let c = projected[next];
            let turn = cross_2d(a, b, c) * winding;

            if turn.abs() <= 1.0e-12 {
                remaining.remove(index);

                continue 'triangulate;
            }

            if turn < 0.0 {
                continue;
            }

            let contains_vertex = remaining.iter().any(|&vertex| {
                vertex != previous
                    && vertex != current
                    && vertex != next
                    && cross_2d(a, b, projected[vertex]) * winding >= -1.0e-12
                    && cross_2d(b, c, projected[vertex]) * winding >= -1.0e-12
                    && cross_2d(c, a, projected[vertex]) * winding >= -1.0e-12
            });

            if !contains_vertex {
                triangles.push([previous, current, next]);
                remaining.remove(index);

                continue 'triangulate;
            }
        }

        return Vec::new();
    }

    if remaining.len() == 3 {
        triangles.push([remaining[0], remaining[1], remaining[2]]);
    }

    triangles
}

fn cross_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[1] - a[1]).mul_add(-(c[0] - a[0]), (b[0] - a[0]) * (c[1] - a[1]))
}
