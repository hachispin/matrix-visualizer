//! Colours used by the graph and its plotted shapes.

use three_d::Srgba;

/// For some reason, [`three_d::ClearState`] wants normalized RGB values instead of [`Srgba`].
///
/// This is #F8FAFC, by the way.
pub const BACKGROUND: (f32, f32, f32) = (0.972_549, 0.980_392_16, 0.988_235_3);

/// Not to be confused with [`AXES`].
pub const GRID: Srgba = Srgba::new_opaque(0xBB, 0xC9, 0xD8);

pub const AXES: Srgba = Srgba::new_opaque(0x47, 0x55, 0x69);
pub const LABELS: Srgba = Srgba::new_opaque(0x33, 0x41, 0x55);

/// Possible colours that shapes can have.
pub const SHAPE_COLORS: [Srgba; 6] = [
    Srgba::new_opaque(0x00, 0x6D, 0xAA),
    Srgba::new_opaque(0xC4, 0x51, 0x00),
    Srgba::new_opaque(0x00, 0x7F, 0x68),
    Srgba::new_opaque(0x9B, 0x4F, 0x9A),
    Srgba::new_opaque(0x9B, 0x6B, 0x00),
    Srgba::new_opaque(0xC0, 0x39, 0x5A),
];
