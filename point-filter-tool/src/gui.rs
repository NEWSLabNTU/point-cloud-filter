use generic_point_filter::Config;
use kiss3d::{
    camera::{ArcBall, Camera},
    nalgebra as na30,
    planar_camera::PlanarCamera,
    post_processing::PostProcessingEffect,
    window::{State, Window},
};
use kiss3d_utils::plot::WindowPlotExt as _;
use nalgebra as na32;

#[derive(Debug, Clone)]
pub struct GuiMsg {
    pub foreground_points: Vec<na32::Point3<f32>>,
    pub background_points: Vec<na32::Point3<f32>>,
}

#[derive(Debug)]
pub struct Gui {
    camera: ArcBall,
    config: Config,
    lidar_tf: Option<na30::Isometry3<f32>>,
    ground_tf: Option<na30::Isometry3<f32>>,
    foreground_points: Vec<na30::Point3<f32>>,
    background_points: Vec<na30::Point3<f32>>,
    rx: flume::Receiver<GuiMsg>,
}

impl Gui {
    pub fn new(config: &Config, rx: flume::Receiver<GuiMsg>) -> Self {
        let mut camera = ArcBall::new(
            na30::Point3::new(0.0, -80.0, 32.0),
            na30::Point3::new(0.0, 0.0, 0.0),
        );
        camera.set_up_axis(na30::Vector3::new(0.0, 0.0, 1.0));

        let convert_isometry3 = |from: &na32::Isometry3<f32>| {
            let na32::Isometry3 {
                rotation,
                translation,
            } = from;
            let na32::coordinates::XYZ { x, y, z } = **translation;
            let na32::coordinates::IJKW { i, j, k, w } = ***rotation;

            na30::Isometry3 {
                rotation: na30::Unit::new_normalize(na30::Quaternion::new(w, i, j, k)),
                translation: na30::Translation3::new(x, y, z),
            }
        };
        let lidar_tf = config
            .lidar_filter
            .as_ref()
            .map(|conf| convert_isometry3(&conf.transform));
        let ground_tf = config
            .ground_filter
            .as_ref()
            .map(|conf| convert_isometry3(&conf.transform));

        Self {
            config: config.clone(),
            camera,
            foreground_points: vec![],
            background_points: vec![],
            rx,
            lidar_tf,
            ground_tf,
        }
    }

    pub fn update(&mut self, msg: GuiMsg) {
        self.foreground_points = msg
            .foreground_points
            .into_iter()
            .map(|pt| {
                let na32::coordinates::XYZ { x, y, z } = *pt;
                na30::Point3::new(x, y, z)
            })
            .collect();
        self.background_points = msg
            .background_points
            .into_iter()
            .map(|pt| {
                let na32::coordinates::XYZ { x, y, z } = *pt;
                na30::Point3::new(x, y, z)
            })
            .collect();
    }

    fn render(&self, window: &mut Window) {
        window.draw_axes(na30::Isometry3::<f32>::identity(), 1.0);

        // Draw axis
        window.draw_axes(na30::Isometry3::identity(), 0.1);

        if let Some(tf) = &self.lidar_tf {
            window.draw_axes(tf.clone(), 0.1);
        }

        if let Some(tf) = &self.ground_tf {
            window.draw_axes(tf.clone(), 0.1);
        }

        // draw filter
        if let Some(filter) = &self.config.range_filter {
            filter.render_kiss3d(window);
        }

        // draw point cloud
        self.background_points.iter().for_each(|point| {
            window.draw_point(point, &na30::Point3::new(0.5, 0.5, 0.5));
        });
        self.foreground_points.iter().for_each(|point| {
            window.draw_point(point, &na30::Point3::new(0.0, 1.0, 0.0));
        });
    }

    // fn lidar_tf(&self) -> Option<&na30::Isometry3<f32>> {
    //     Some(&self.config.lidar_filter?.transform)
    // }

    // fn ground_tf(&self) -> Option<&na30::Isometry3<f32>> {
    //     Some(&self.config.ground_filter?.transform)
    // }
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
