//! Colours used by the graph and its plotted shapes.

use three_d::Srgba;

/// Background behind the plotting area (#F8FAFC).
pub const BACKGROUND: Srgba = Srgba::new_opaque(0xF8, 0xFA, 0xFC);
/// Subtle tint inside the plotting box (#E8EFF5).
pub const PLOT_AREA: Srgba = Srgba::new_opaque(0xE8, 0xEF, 0xF5);
/// Minor grid lines (#BBC9D8).
pub const MINOR_GRID: Srgba = Srgba::new_opaque(0xBB, 0xC9, 0xD8);
/// Major grid lines (#93A7BA).
pub const MAJOR_GRID: Srgba = Srgba::new_opaque(0x93, 0xA7, 0xBA);
/// Outline of the 3D plotting box (#9AA8B8).
pub const CUBE_OUTLINE: Srgba = Srgba::new_opaque(0x9A, 0xA8, 0xB8);
/// Axes and arrowheads (#475569).
pub const AXES: Srgba = Srgba::new_opaque(0x47, 0x55, 0x69);
/// Numbers and labels (#334155).
pub const LABELS: Srgba = Srgba::new_opaque(0x33, 0x41, 0x55);

/// Colours assigned to plotted shapes in order: blue, orange, teal, purple,
/// gold, and rose.
pub const SHAPE_COLORS: [Srgba; 6] = [
    Srgba::new_opaque(0x00, 0x6D, 0xAA),
    Srgba::new_opaque(0xC4, 0x51, 0x00),
    Srgba::new_opaque(0x00, 0x7F, 0x68),
    Srgba::new_opaque(0x9B, 0x4F, 0x9A),
    Srgba::new_opaque(0x9B, 0x6B, 0x00),
    Srgba::new_opaque(0xC0, 0x39, 0x5A),
];
