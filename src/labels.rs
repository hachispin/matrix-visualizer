//! Textured grid labels, with cylindrical billboards for the vertical axis.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "Raster dimensions and bounded grid coordinates fit in these types"
)]

use anyhow::{Result, anyhow};
use fontdue::{
    Font,
    FontSettings,
    layout::{CoordinateSystem, Layout, TextStyle},
};
use three_d::{
    Camera,
    ColorMaterial,
    Context,
    CpuMaterial,
    CpuMesh,
    CpuTexture,
    Gm,
    Mesh,
    Object,
    TextureData,
    prelude::*,
};

use crate::{grid, palette};

const FONT_SIZE: f32 = 64.0;
const NUMBER_TEXT_HEIGHT: f32 = 0.2;
const AXIS_TEXT_HEIGHT: f32 = 0.3;
const AXIS_GAP: f32 = 0.18;
const NUMBER_GAP: f32 = 0.08;
// Lift the XY textures just enough to avoid fighting with the grid's depth.
const PLANE_HEIGHT: f32 = 0.02;

struct VerticalLabel {
    object: Gm<Mesh, ColorMaterial>,
    height: f32,
    half_width: f32,
}

pub struct Labels {
    font: Font,
    number_font: Font,
    plane: Vec<Gm<Mesh, ColorMaterial>>,
    vertical: Vec<VerticalLabel>,
    centre: Vec3,
    magnification: f32,
    facing: Mat4,
}

impl Labels {
    pub fn new(ctx: &Context, centre: Vec3, magnification: f32) -> Result<Self> {
        let font = Font::from_bytes(
            include_bytes!("../assets/IBMPlexSans-Bold.ttf") as &[u8],
            FontSettings::default(),
        )
        .map_err(|e| anyhow!("{e}"))?;
        let number_font = Font::from_bytes(
            include_bytes!("../assets/IBMPlexSans-SemiBold.ttf") as &[u8],
            FontSettings::default(),
        )
        .map_err(|e| anyhow!("{e}"))?;

        let mut labels = Self {
            font,
            number_font,
            plane: Vec::new(),
            vertical: Vec::new(),
            centre,
            magnification,
            facing: Mat4::from_angle_x(degrees(90.0)),
        };

        labels.rebuild(ctx)?;

        Ok(labels)
    }

    /// Rebuild text only when its values change; camera motion only changes Z transforms.
    pub fn update(
        &mut self,
        ctx: &Context,
        camera: &Camera,
        centre: Vec3,
        magnification: f32,
    ) -> Result<()> {
        if self.centre != centre || self.magnification.to_bits() != magnification.to_bits() {
            self.centre = centre;
            self.magnification = magnification;
            self.rebuild(ctx)?;
        }

        if let Some(facing) = cylindrical_facing(camera.position()) {
            self.facing = facing;
        }

        for label in &mut self.vertical {
            label.object.set_transformation(
                Mat4::from_translation(vec3(0.0, 0.0, label.height))
                    * self.facing
                    * Mat4::from_translation(vec3(AXIS_GAP + label.half_width, 0.0, 0.0)),
            );
        }

        Ok(())
    }

    pub fn objects(&self) -> impl Iterator<Item = &dyn Object> {
        self.plane.iter().map(|object| object as &dyn Object).chain(
            self.vertical
                .iter()
                .map(|label| &label.object as &dyn Object),
        )
    }

    fn rebuild(&mut self, ctx: &Context) -> Result<()> {
        self.plane.clear();
        self.vertical.clear();

        let step = grid::interval(self.magnification);
        let spacing = step * self.magnification;
        let limit = (grid::GRID_SIZE / spacing).floor() as i32;
        let mut horizontal = Vec::new();
        let mut max_width = 0.0_f32;

        for tick in -limit..=limit {
            // Don't add labels at origin.
            if tick == 0 {
                continue;
            }

            let position = tick as f32 * spacing;
            let value = tick as f32 * step;
            let values = vec3(value, value, value) + self.centre;

            let (mut object, half_width) = text_gm(
                ctx,
                &self.number_font,
                &format_value(values.x, step),
                NUMBER_TEXT_HEIGHT,
            )?;
            object.set_transformation(Mat4::from_translation(vec3(position, -0.3, PLANE_HEIGHT)));
            max_width = max_width.max(2.0 * half_width);
            horizontal.push((tick, object));

            self.add_plane_label(
                ctx,
                &format_value(values.y, step),
                vec3(0.0, position, PLANE_HEIGHT),
                true,
                NUMBER_TEXT_HEIGHT,
            )?;

            self.add_vertical_label(
                ctx,
                &format_value(values.z, step),
                position,
                NUMBER_TEXT_HEIGHT,
            )?;
        }

        // Leave room for the widest X label, keeping the chosen ticks symmetric about zero.
        let stride = ((max_width + NUMBER_GAP) / spacing).ceil().max(1.0) as i32;
        self.plane.extend(
            horizontal
                .into_iter()
                .filter(|(tick, _)| tick % stride == 0)
                .map(|(_, object)| object),
        );

        let end = grid::GRID_SIZE + 0.4;

        self.add_plane_label(
            ctx,
            "x",
            vec3(end, 0.0, PLANE_HEIGHT),
            false,
            AXIS_TEXT_HEIGHT,
        )?;
        self.add_plane_label(
            ctx,
            "y",
            vec3(0.0, end, PLANE_HEIGHT),
            false,
            AXIS_TEXT_HEIGHT,
        )?;
        self.add_vertical_label(ctx, "z", end, AXIS_TEXT_HEIGHT)?;

        Ok(())
    }

    fn add_plane_label(
        &mut self,
        ctx: &Context,
        text: &str,
        mut position: Vec3,
        align_right: bool,
        text_height: f32,
    ) -> Result<()> {
        let (mut object, half_width) = text_gm(ctx, self.label_font(text), text, text_height)?;

        if align_right {
            position.x -= AXIS_GAP + half_width;
        }

        object.set_transformation(Mat4::from_translation(position));
        self.plane.push(object);

        Ok(())
    }

    fn add_vertical_label(
        &mut self,
        ctx: &Context,
        text: &str,
        height: f32,
        text_height: f32,
    ) -> Result<()> {
        let (object, half_width) = text_gm(ctx, self.label_font(text), text, text_height)?;

        self.vertical.push(VerticalLabel {
            object,
            height,
            half_width,
        });

        Ok(())
    }

    fn label_font(&self, text: &str) -> &Font {
        if text.bytes().any(|byte| byte.is_ascii_digit()) {
            &self.number_font
        } else {
            &self.font
        }
    }
}

/// Local text Y remains world Z. Only the horizontal normal follows the camera.
fn cylindrical_facing(camera_position: Vec3) -> Option<Mat4> {
    let horizontal = vec3(camera_position.x, camera_position.y, 0.0);

    if horizontal.magnitude2() < 1.0e-8 {
        return None;
    }

    let normal = horizontal.normalize();
    let right = Vec3::unit_z().cross(normal);

    Some(Mat4::from_cols(
        right.extend(0.0),
        Vec3::unit_z().extend(0.0),
        normal.extend(0.0),
        vec4(0.0, 0.0, 0.0, 1.0),
    ))
}

fn format_value(value: f32, step: f32) -> String {
    let precision = (1.0 - step.log10().floor()).clamp(0.0, 6.0) as usize;
    let text = format!("{value:.precision$}");

    let text = if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.')
    } else {
        &text
    };

    if text == "-0" {
        "0".to_owned()
    } else {
        text.to_owned()
    }
}

fn rasterize_text(font: &Font, text: &str) -> (usize, usize, Vec<u8>) {
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.append(&[font], &TextStyle::new(text, FONT_SIZE, 0));

    // Crop to the ink bounds so leading ascent space does not offset the label.
    let glyphs: Vec<_> = layout
        .glyphs()
        .iter()
        .filter(|g| g.width > 0 && g.height > 0)
        .collect();

    let left = glyphs.iter().map(|g| g.x.floor() as i32).min().unwrap_or(0);
    let top = glyphs.iter().map(|g| g.y.floor() as i32).min().unwrap_or(0);

    let right = glyphs
        .iter()
        .map(|g| (g.x.floor() + g.width as f32) as i32)
        .max()
        .unwrap_or(left);

    let bottom = glyphs
        .iter()
        .map(|g| (g.y.floor() + g.height as f32) as i32)
        .max()
        .unwrap_or(top);

    let width = (right - left) as usize + 4;
    let height = (bottom - top) as usize + 4;
    let mut pixels = vec![0; width * height];

    for glyph in glyphs {
        let (_, bitmap) = font.rasterize_config(glyph.key);
        let x = (glyph.x.floor() as i32 - left) as usize + 2;
        let y = (glyph.y.floor() as i32 - top) as usize + 2;

        for row in 0..glyph.height {
            for column in 0..glyph.width {
                let pixel = &mut pixels[(y + row) * width + x + column];
                *pixel = (*pixel).max(bitmap[row * glyph.width + column]);
            }
        }
    }

    (width, height, pixels)
}

fn text_gm(
    ctx: &Context,
    font: &Font,
    text: &str,
    text_height: f32,
) -> Result<(Gm<Mesh, ColorMaterial>, f32)> {
    let (width, height, pixels) = rasterize_text(font, text);

    let texture = CpuTexture {
        width: u32::try_from(width)?,
        height: u32::try_from(height)?,
        data: TextureData::RgbaU8(
            pixels
                .into_iter()
                .map(|alpha| [255, 255, 255, alpha])
                .collect(),
        ),
        ..Default::default()
    };

    let material = ColorMaterial::new_transparent(
        ctx,
        &CpuMaterial {
            albedo: palette::LABELS,
            albedo_texture: Some(texture),
            ..Default::default()
        },
    );

    let half_width = width as f32 * text_height / FONT_SIZE / 2.0;
    let half_height = height as f32 * text_height / FONT_SIZE / 2.0;

    let mut mesh = CpuMesh::square();
    mesh.transform(Mat4::from_nonuniform_scale(half_width, half_height, 1.0))?;

    Ok((Gm::new(Mesh::new(ctx, &mesh), material), half_width))
}
