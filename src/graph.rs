//! Graph objects and their shared change checks.

use std::iter;

use anyhow::Result;
use three_d::{Camera, ColorMaterial, Context, CpuMesh, Gm, Mesh, prelude::*};

use crate::{
    grid,
    labels::Labels,
    palette,
    polygons::{Polygon, Polygons},
    shapes::{LineSegment, Point, Points, Segments},
    ui::Ui,
    utils,
};

const ORIGIN_RADIUS: f32 = 0.1;

struct State {
    centre: Vec3,
    magnification: f32,
    heading: u32,
    points: Vec<Point>,
    segments: Vec<LineSegment>,
    polygons: Vec<Polygon>,
    transformation: Mat3,
    animation_endpoints: Option<(Mat3, Mat3)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            centre: Vec3::zero(),
            magnification: 1.0,
            heading: 0,
            points: Vec::new(),
            segments: Vec::new(),
            polygons: Vec::new(),
            transformation: Mat3::identity(),
            animation_endpoints: None,
        }
    }
}

pub struct Graph {
    grid: Gm<Mesh, ColorMaterial>,
    labels: Labels,
    origin: Gm<Mesh, ColorMaterial>,
    origin_visible: bool,
    points: Points,
    animation_points: [Points; 2],
    segments: Segments,
    animation_segments: [Segments; 2],
    polygons: Polygons,
    animation_polygons: [Polygons; 2],
    state: State,
}

impl Graph {
    pub fn new(ctx: &Context, camera: &Camera) -> Result<Self> {
        let grid = Gm::new(grid::mesh(ctx, 1.0)?, ColorMaterial::default());

        let mut labels = Labels::new(ctx, Vec3::zero(), 1.0)?;
        labels.face_camera(camera);

        // origin marker
        let mut origin = Gm::new(
            Mesh::new(ctx, &CpuMesh::sphere(16)),
            ColorMaterial {
                color: palette::AXES,
                ..Default::default()
            },
        );

        origin.set_transformation(Mat4::from_scale(ORIGIN_RADIUS));

        Ok(Self {
            grid,
            labels,
            origin,
            origin_visible: true,
            points: Points::default(),
            animation_points: std::array::from_fn(|_| Points::translucent()),
            segments: Segments::default(),
            animation_segments: std::array::from_fn(|_| Segments::translucent()),
            polygons: Polygons::default(),
            animation_polygons: std::array::from_fn(|_| Polygons::translucent()),
            state: State {
                heading: camera_heading(camera),
                ..Default::default()
            },
        })
    }

    /// Updates certain components of the graph depending on what has changed.
    ///
    /// For instance, if the transformations have changed, points need to be updated. However, other
    /// components such as labels don't need to change (be redrawn). This helps with performance.
    pub fn update(
        &mut self,
        ctx: &Context,
        camera: &Camera,
        centre: Vec3,
        magnification: f32,
        ui: &Ui,
    ) -> Result<()> {
        let zoom_changed = magnification.to_bits() != self.state.magnification.to_bits();
        let view_changed = zoom_changed || centre != self.state.centre;
        let matrix_changed = ui.transformation != self.state.transformation;
        let animation_endpoints = ui.animation_endpoints();
        let endpoints_changed = animation_endpoints != self.state.animation_endpoints;
        let points_changed = ui.points != self.state.points;
        let segments_changed = ui.segments != self.state.segments;
        let polygons_changed = ui.polygons != self.state.polygons;
        let heading = camera_heading(camera);

        if zoom_changed {
            self.grid.geometry = grid::mesh(ctx, magnification)?;
        }

        if view_changed {
            self.labels.rebuild(ctx, centre, magnification)?;
        }

        if view_changed || heading != self.state.heading {
            self.labels.face_camera(camera);
        }

        if view_changed {
            let position = utils::graph_to_world(Vec3::zero(), centre, magnification);

            self.origin.set_transformation(
                Mat4::from_translation(position) * Mat4::from_scale(ORIGIN_RADIUS),
            );

            self.origin_visible = grid::contains(position, ORIGIN_RADIUS);
        }

        if points_changed || view_changed || matrix_changed {
            self.points
                .update(ctx, &ui.points, ui.transformation, centre, magnification);
        }

        if (points_changed || view_changed || endpoints_changed)
            && let Some((start, end)) = animation_endpoints
        {
            for (points, transformation) in self.animation_points.iter_mut().zip([start, end]) {
                points.update(ctx, &ui.points, transformation, centre, magnification);
            }
        }

        if segments_changed || view_changed || matrix_changed {
            self.segments
                .update(ctx, &ui.segments, ui.transformation, centre, magnification)?;
        }

        if (segments_changed || view_changed || endpoints_changed)
            && let Some((start, end)) = animation_endpoints
        {
            for (segments, transformation) in self.animation_segments.iter_mut().zip([start, end]) {
                segments.update(ctx, &ui.segments, transformation, centre, magnification)?;
            }
        }

        if polygons_changed || view_changed || matrix_changed {
            self.polygons
                .update(ctx, &ui.polygons, ui.transformation, centre, magnification)?;
        }

        if (polygons_changed || view_changed || endpoints_changed)
            && let Some((start, end)) = animation_endpoints
        {
            for (polygons, transformation) in self.animation_polygons.iter_mut().zip([start, end]) {
                polygons.update(ctx, &ui.polygons, transformation, centre, magnification)?;
            }
        }

        self.state.centre = centre;
        self.state.magnification = magnification;
        self.state.heading = heading;
        self.state.transformation = ui.transformation;
        self.state.animation_endpoints = animation_endpoints;

        if points_changed {
            self.state.points.clone_from(&ui.points);
        }

        if segments_changed {
            self.state.segments.clone_from(&ui.segments);
        }

        if polygons_changed {
            self.state.polygons.clone_from(&ui.polygons);
        }

        Ok(())
    }

    pub fn objects(&self) -> impl Iterator<Item = &Gm<Mesh, ColorMaterial>> {
        iter::once(&self.grid)
            .chain(self.origin_visible.then_some(&self.origin))
            .chain(self.points.objects())
            .chain(self.segments.objects())
            .chain(self.polygons.objects())
            .chain(
                self.animation_points
                    .iter()
                    .filter(|_| self.state.animation_endpoints.is_some())
                    .flat_map(Points::objects),
            )
            .chain(
                self.animation_segments
                    .iter()
                    .filter(|_| self.state.animation_endpoints.is_some())
                    .flat_map(Segments::objects),
            )
            .chain(
                self.animation_polygons
                    .iter()
                    .filter(|_| self.state.animation_endpoints.is_some())
                    .flat_map(Polygons::objects),
            )
            .chain(self.labels.objects())
    }
}

fn camera_heading(camera: &Camera) -> u32 {
    let position = camera.position();

    camera.position().y.atan2(position.x).to_bits()
}
