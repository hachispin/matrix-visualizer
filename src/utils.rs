//! Place to put general utility functions.

use anyhow::Result;
use three_d::{
    ColorMaterial,
    Context,
    CpuMaterial,
    CpuMesh,
    Gm,
    Indices,
    Mesh,
    Positions,
    prelude::*,
};

use crate::grid;

/// Merges two meshes into one mesh.
pub fn merge_meshes(a: CpuMesh, b: CpuMesh) -> Result<CpuMesh> {
    let a_len = a.positions.len();
    let b_len = b.positions.len();

    let a_len_u32 = u32::try_from(a_len)?;
    let b_len_u32 = u32::try_from(b_len)?;

    let a_indices = a
        .indices
        .into_u32()
        .unwrap_or_else(|| (0..(a_len_u32)).collect());

    let b_indices = b
        .indices
        .into_u32()
        .unwrap_or_else(|| (0..(b_len_u32)).collect());

    let a_colors = a.colors.unwrap_or_else(|| vec![Srgba::WHITE; a_len]);
    let b_colors = b.colors.unwrap_or_else(|| vec![Srgba::WHITE; b_len]);

    let indices = a_indices
        .into_iter()
        .chain(b_indices.into_iter().map(|i| i + a_len_u32))
        .collect();

    let colors = a_colors.into_iter().chain(b_colors).collect();

    let positions = a
        .positions
        .into_f32()
        .into_iter()
        .chain(b.positions.into_f32())
        .collect();

    Ok(CpuMesh {
        positions: Positions::F32(positions),
        indices: Indices::U32(indices),
        colors: Some(colors),
        ..Default::default()
    })
}

/// Draws a 2D line with a width.
pub fn draw_2d_line(
    positions: &mut Vec<Vector3<f32>>,
    indices: &mut Vec<u32>,
    p1: Vector3<f32>,
    p2: Vector3<f32>,
    line_width: f32,
) -> Result<()> {
    // add width perpendicular to the line
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;

    let length = dx.hypot(dy);

    if length == 0.0 {
        return Ok(());
    }

    let offset = vec3(-dy, dx, 0.0) * (line_width / (2.0 * length));
    let base = u32::try_from(positions.len())?;

    positions.push(p1 - offset);
    positions.push(p1 + offset);
    positions.push(p2 - offset);
    positions.push(p2 + offset);

    // winding order: doesn't really matter here since backface culling is usually
    // disabled but this order ensures both triangle of the quad are facing the same way.
    indices.extend([base, base + 2, base + 1, base + 1, base + 2, base + 3]);

    Ok(())
}

/// Map graph coordinates into the fixed world-space grid.
pub fn graph_to_world(point: Vec3, centre: Vec3, magnification: f32) -> Vec3 {
    (point - centre) * magnification
}

/// Map world coordinates back into graph coordinates.
pub fn world_to_graph(point: Vec3, centre: Vec3, magnification: f32) -> Vec3 {
    point / magnification + centre
}

pub fn update_mesh(
    ctx: &Context,
    object: &mut Option<Gm<Mesh, ColorMaterial>>,
    positions: Vec<Vec3>,
    indices: Vec<u32>,
    color: Srgba,
) {
    if indices.is_empty() {
        *object = None;

        return;
    }

    let mesh = CpuMesh {
        positions: Positions::F32(positions),
        indices: Indices::U32(indices),
        ..CpuMesh::default()
    };

    if let Some(object) = object {
        object.geometry = Mesh::new(ctx, &mesh);
        object.material.color = color;
    } else {
        let material = if color.a < 255 {
            ColorMaterial::new_transparent(ctx, &CpuMaterial::default())
        } else {
            ColorMaterial::default()
        };
        let mut rendered = Gm::new(Mesh::new(ctx, &mesh), material);
        rendered.material.color = color;

        *object = Some(rendered);
    }
}

/// Widen before mapping and clipping, including for vertices far outside the visible grid.
pub fn world_position(
    position: Vec3,
    transformation: Mat3,
    centre: Vec3,
    magnification: f32,
) -> Vector3<f64> {
    Vector3::from(std::array::from_fn::<_, 3, _>(|row| {
        let image: f64 = (0..3)
            .map(|column| f64::from(transformation[column][row]) * f64::from(position[column]))
            .sum();

        (image - f64::from(centre[row])) * f64::from(magnification)
    }))
}

pub fn clip_edge(mut start: Vector3<f64>, mut end: Vector3<f64>) -> Option<(Vec3, Vec3)> {
    for axis in 0..3 {
        for sign in [-1.0, 1.0] {
            let start_distance = f64::mul_add(sign, -start[axis], f64::from(grid::GRID_SIZE));
            let end_distance = f64::mul_add(sign, -end[axis], f64::from(grid::GRID_SIZE));

            if start_distance < 0.0 && end_distance < 0.0 {
                return None;
            }

            if (start_distance >= 0.0) != (end_distance >= 0.0) {
                let amount = start_distance / (start_distance - end_distance);
                let mut intersection = start + (end - start) * amount;
                intersection[axis] = sign * f64::from(grid::GRID_SIZE);

                if start_distance < 0.0 {
                    start = intersection;
                } else {
                    end = intersection;
                }
            }
        }
    }

    Some((
        start.map(|value| value as f32),
        end.map(|value| value as f32),
    ))
}

pub fn append_edge(
    positions: &mut Vec<Vec3>,
    indices: &mut Vec<u32>,
    start: Vec3,
    end: Vec3,
) -> Result<()> {
    let direction = end - start;

    if direction.magnitude2() < 1.0e-12 {
        return Ok(());
    }

    let direction = direction.normalize();
    let reference = if direction.z.abs() < 0.9 {
        Vec3::unit_z()
    } else {
        Vec3::unit_y()
    };
    let right = direction.cross(reference).normalize() * 0.025;
    let up = direction.cross(right);
    let base = u32::try_from(positions.len())?;

    for side in 0..8 {
        let angle = side as f32 * std::f32::consts::TAU / 8.0;
        let offset = right * angle.cos() + up * angle.sin();
        positions.extend([start + offset, end + offset]);

        let current = base + 2 * side;
        let next = base + 2 * ((side + 1) % 8);
        indices.extend([current, current + 1, next, next, current + 1, next + 1]);
    }

    Ok(())
}
