//! Place to put general utility functions.

use three_d::{CpuMesh, Indices, Positions, Srgba, Vector3, vec3};

/// Merges two meshes into one mesh.
pub fn merge_meshes(a: CpuMesh, b: CpuMesh) -> CpuMesh {
    let mut positions = a.positions.into_f32();
    let other_positions = b.positions.into_f32();

    let a_count = positions.len();
    let b_count = other_positions.len();
    let offset = u32::try_from(a_count).unwrap();
    let b_count_u32 = u32::try_from(b_count).unwrap();

    // no indices = order of vertices
    let mut indices = a
        .indices
        .into_u32()
        .unwrap_or_else(|| (0..offset).collect());

    let other_indices = b
        .indices
        .into_u32()
        .unwrap_or_else(|| (0..b_count_u32).collect());

    indices.extend(other_indices.into_iter().map(|i| i + offset));

    let mut colors = a.colors.unwrap_or_else(|| vec![Srgba::WHITE; a_count]);

    colors.extend(b.colors.unwrap_or_else(|| vec![Srgba::WHITE; b_count]));

    positions.extend(other_positions);

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
    let base = u32::try_from(positions.len()).unwrap();

    positions.push(p1 - offset);
    positions.push(p1 + offset);
    positions.push(p2 - offset);
    positions.push(p2 + offset);

    // winding order: doesn't really matter here since backface culling is usually
    // disabled but this order ensures both triangle of the quad are facing the same way.
    indices.extend([base, base + 2, base + 1, base + 1, base + 2, base + 3]);
}
