use crate::config::Config;
use kiss3d::{
    camera::{ArcBall, Camera},
    planar_camera::PlanarCamera,
    post_processing::PostProcessingEffect,
    window::{State, Window},
};
use kiss3d_utils::plot::WindowPlotExt as _;
use nalgebra as na;

#[derive(Debug, Clone)]
pub struct GuiMsg {
    pub foreground_points: Vec<na::Point3<f32>>,
    pub background_points: Vec<na::Point3<f32>>,
}

#[derive(Debug)]
pub struct Gui {
    camera: ArcBall,
    config: Config,
    lidar_tf: na::Isometry3<f32>,
    ground_tf: na::Isometry3<f32>,
    foreground_points: Vec<na::Point3<f32>>,
    background_points: Vec<na::Point3<f32>>,
    rx: flume::Receiver<GuiMsg>,
}

impl Gui {
    pub fn new(
        config: &Config,
        lidar_tf: na::Isometry3<f32>,
        ground_tf: na::Isometry3<f32>,
        rx: flume::Receiver<GuiMsg>,
    ) -> Self {
        let mut camera = ArcBall::new(
            na::Point3::new(0.0, -80.0, 32.0),
            na::Point3::new(0.0, 0.0, 0.0),
        );
        camera.set_up_axis(na::Vector3::new(0.0, 0.0, 1.0));

        Self {
            config: config.clone(),
            camera,
            foreground_points: vec![],
            background_points: vec![],
            lidar_tf,
            ground_tf,
            rx,
        }
    }

    pub fn update(&mut self, msg: GuiMsg) {
        self.foreground_points = msg.foreground_points;
        self.background_points = msg.background_points;
    }

    fn render(&self, window: &mut Window) {
        window.draw_axes(na::Isometry3::<f32>::identity(), 1.0);

        // Draw axis
        window.draw_axes(na::Isometry3::identity(), 0.1);
        window.draw_axes(self.lidar_tf, 0.1);
        window.draw_axes(self.ground_tf, 0.1);

        // draw filter
        if let Some(rules) = &self.config.range_filter_rules {
            rules.render_kiss3d(window);
        }

        // draw point cloud
        self.background_points.iter().for_each(|point| {
            window.draw_point(point, &na::Point3::new(0.5, 0.5, 0.5));
        });
        self.foreground_points.iter().for_each(|point| {
            window.draw_point(point, &na::Point3::new(0.0, 1.0, 0.0));
        });
    }
}

impl State for Gui {
    fn step(&mut self, window: &mut Window) {
        use flume::TryRecvError as E;

        match self.rx.try_recv() {
            Ok(msg) => self.update(msg),
            Err(E::Disconnected) => {
                window.close();
                return;
            }
            Err(E::Empty) => {}
        };

        self.render(window);
    }

    #[allow(clippy::type_complexity)]
    fn cameras_and_effect(
        &mut self,
    ) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn PostProcessingEffect>,
    ) {
        (Some(&mut self.camera), None, None)
    }
}
