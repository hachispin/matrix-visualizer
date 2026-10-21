//! Graph objects and their shared change checks.

use std::iter;

use anyhow::Result;
use three_d::{Camera, ColorMaterial, Context, CpuMesh, Gm, Mesh, prelude::*};

use crate::{
    grid,
    labels::Labels,
    palette,
    shapes::{Point, Points},
    ui::Ui,
    utils,
};

const ORIGIN_RADIUS: f32 = 0.1;

struct State {
    centre: Vec3,
    magnification: f32,
    heading: u32,
    points: Vec<Point>,
    transformation: Mat3,
}

impl Default for State {
    fn default() -> Self {
        Self {
            centre: Vec3::zero(),
            magnification: 1.0,
            heading: 0,
            points: Vec::new(),
            transformation: Mat3::identity(),
        }
    }
}

pub struct Graph {
    grid: Gm<Mesh, ColorMaterial>,
    labels: Labels,
    origin: Gm<Mesh, ColorMaterial>,
    origin_visible: bool,
    points: Points,
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
        let points_changed = ui.points != self.state.points;
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

        self.state.centre = centre;
        self.state.magnification = magnification;
        self.state.heading = heading;
        self.state.transformation = ui.transformation;

        if points_changed {
            self.state.points.clone_from(&ui.points);
        }

        Ok(())
    }

    pub fn objects(&self) -> impl Iterator<Item = &Gm<Mesh, ColorMaterial>> {
        iter::once(&self.grid)
            .chain(self.origin_visible.then_some(&self.origin))
            .chain(self.points.objects())
            .chain(self.labels.objects())
    }
}

fn camera_heading(camera: &Camera) -> u32 {
    let position = camera.position();

    camera.position().y.atan2(position.x).to_bits()
}
