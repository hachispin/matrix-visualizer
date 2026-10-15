//! Place to put general utility functions.

use three_d::{CpuMesh, Indices, Positions, prelude::*};

/// Merges two meshes into one mesh.
pub fn merge_meshes(a: CpuMesh, b: CpuMesh) -> CpuMesh {
    let a_len = a.positions.len();
    let b_len = b.positions.len();

    #[expect(
        clippy::cast_possible_truncation,
        reason = "surely one does not have a mesh with u32::MAX + 1 positions?"
    )]
    let (a_len_u32, b_len_u32) = (a_len as u32, b_len as u32);

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

    CpuMesh {
        positions: Positions::F32(positions),
        indices: Indices::U32(indices),
        colors: Some(colors),
        ..Default::default()
    }
}

/// Draws a 2D line with a width.
pub fn draw_2d_line(
    positions: &mut Vec<Vector3<f32>>,
    indices: &mut Vec<u32>,
    p1: Vector3<f32>,
    p2: Vector3<f32>,
    line_width: f32,
) {
    // add width perpendicular to the line
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;

    let length = dx.hypot(dy);

    if length == 0.0 {
        return;
    }

    let offset = vec3(-dy, dx, 0.0) * (line_width / (2.0 * length));

    #[expect(
        clippy::cast_possible_truncation,
        reason = "will never be realistically truncated"
    )]
    let base = positions.len() as u32;

    positions.push(p1 - offset);
    positions.push(p1 + offset);
    positions.push(p2 - offset);
    positions.push(p2 + offset);

    // winding order: doesn't really matter here since backface culling is usually
    // disabled but this order ensures both triangle of the quad are facing the same way.
    indices.extend([base, base + 2, base + 1, base + 1, base + 2, base + 3]);
}

// will probably use later
//
// pub fn graph_to_world(point: Vec3, centre: Vec3, magnification: f32) -> Vec3 {
//     (point - centre) * magnification
// }
//
// pub fn world_to_graph(point: Vec3, centre: Vec3, magnification: f32) -> Vec3 {
//     point / magnification + centre
// }
