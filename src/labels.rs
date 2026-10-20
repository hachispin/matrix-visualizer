//! Labels for the graph.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "Unlikely to occur"
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
    TextureData,
    prelude::*,
};

use crate::{grid, palette};

const FONT_SIZE: f32 = 64.0;
const NUMBER_TEXT_HEIGHT: f32 = 0.2;
const AXIS_TEXT_HEIGHT: f32 = 0.3;
const AXIS_GAP: f32 = 0.18;
const NUMBER_GAP: f32 = 0.08;
// prevent z-fighting
const PLANE_HEIGHT: f32 = 0.02;

struct VerticalLabel {
    object: Gm<Mesh, ColorMaterial>,
    height: f32,
    half_width: f32,
}

pub struct Labels {
    axis_font: Font,
    number_font: Font,
    plane: Vec<Gm<Mesh, ColorMaterial>>,
    vertical: Vec<VerticalLabel>,
    facing: Mat4,
}

impl Labels {
    pub fn new(ctx: &Context, centre: Vec3, magnification: f32) -> Result<Self> {
        let axis_font = Font::from_bytes(
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
            axis_font,
            number_font,
            plane: Vec::new(),
            vertical: Vec::new(),
            facing: Mat4::from_angle_x(degrees(90.0)),
        };

        labels.rebuild(ctx, centre, magnification)?;

        Ok(labels)
    }

    /// Make z labels face the camera cylindrically.
    pub fn face_camera(&mut self, camera: &Camera) {
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
    }

    pub fn objects(&self) -> impl Iterator<Item = &Gm<Mesh, ColorMaterial>> {
        self.plane
            .iter()
            .chain(self.vertical.iter().map(|label| &label.object))
    }

    /// Rebuild unconditionally; the caller decides when graph state has changed.
    pub fn rebuild(&mut self, ctx: &Context, centre: Vec3, magnification: f32) -> Result<()> {
        self.plane.clear();
        self.vertical.clear();

        let step = grid::interval(magnification);
        let spacing = step * magnification;
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
            let values = vec3(value, value, value) + centre;

            let (mut object, half_width) = text_gm(
                ctx,
                &self.number_font,
                &format_value(values.x, step),
                NUMBER_TEXT_HEIGHT,
            )?;
            object.set_transformation(Mat4::from_translation(vec3(position, -0.3, PLANE_HEIGHT)));
            max_width = max_width.max(2.0 * half_width);
            horizontal.push((tick, object));

            self.plane.push(Self::plane_label(
                ctx,
                &self.number_font,
                &format_value(values.y, step),
                vec3(0.0, position, PLANE_HEIGHT),
                true,
                NUMBER_TEXT_HEIGHT,
            )?);

            self.vertical.push(Self::vertical_label(
                ctx,
                &self.number_font,
                &format_value(values.z, step),
                position,
                NUMBER_TEXT_HEIGHT,
            )?);
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

        self.plane.push(Self::plane_label(
            ctx,
            &self.axis_font,
            "x",
            vec3(end, 0.0, PLANE_HEIGHT),
            false,
            AXIS_TEXT_HEIGHT,
        )?);
        self.plane.push(Self::plane_label(
            ctx,
            &self.axis_font,
            "y",
            vec3(0.0, end, PLANE_HEIGHT),
            false,
            AXIS_TEXT_HEIGHT,
        )?);
        self.vertical.push(Self::vertical_label(
            ctx,
            &self.axis_font,
            "z",
            end,
            AXIS_TEXT_HEIGHT,
        )?);

        Ok(())
    }

    fn plane_label(
        ctx: &Context,
        font: &Font,
        text: &str,
        mut position: Vec3,
        align_right: bool,
        text_height: f32,
    ) -> Result<Gm<Mesh, ColorMaterial>> {
        let (mut object, half_width) = text_gm(ctx, font, text, text_height)?;

        if align_right {
            position.x -= AXIS_GAP + half_width;
        }

        object.set_transformation(Mat4::from_translation(position));
        Ok(object)
    }

    fn vertical_label(
        ctx: &Context,
        font: &Font,
        text: &str,
        height: f32,
        text_height: f32,
    ) -> Result<VerticalLabel> {
        let (object, half_width) = text_gm(ctx, font, text, text_height)?;

        Ok(VerticalLabel {
            object,
            height,
            half_width,
        })
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
