use crate::{
    item::{Item, PlanarBoxFilter},
    FilterProgram,
};
use filter_expr::normal_form::Term;
use kiss3d::window::Window;
use kiss3d_utils::WindowPlotExt as _;
use nalgebra_0_30 as na30;
use nalgebra_0_32 as na32;

impl FilterProgram {
    pub fn render_kiss3d(&self, window: &mut Window) {
        for product in &self.dnf.0 {
            for term in &product.0 {
                let Term { ref ident, invert } = *term;

                let pos_color = na30::Point3::new(0.0, 1.0, 0.0);
                let neg_color = na30::Point3::new(1.0, 0.0, 0.0);
                let color = if invert { neg_color } else { pos_color };

                let item = &self.items[&ident.0];
                match item {
                    Item::Box(filter) => filter.render_kiss3d(window, &color),
                    Item::Intensity(_) => {}
                }
            }
        }
    }
}

impl PlanarBoxFilter {
    pub fn render_kiss3d(&self, window: &mut Window, color: &na30::Point3<f32>) {
        if let Some(pose) = self.pose_3d() {
            let pose: na32::Isometry3<f32> = na32::convert(pose);
            let na32::coordinates::XYZ { x, y, z } = *pose.translation;
            let na32::coordinates::IJKW { i, j, k, w } = **pose.rotation;
            let pose = na30::Isometry3 {
                rotation: na30::Unit::new_normalize(na30::Quaternion::new(w, i, j, k)),
                translation: na30::Translation3::new(x, y, z),
            };
            let extends = self.extends_3d().unwrap();
            window.draw_box(extends, pose, color);
        } else {
            let na32::Isometry2 {
                rotation,
                translation,
            } = self.pose_2d();

            let na32::coordinates::XY { x, y } = *translation.vector;
            let yaw = rotation.angle();

            let pose = na30::Isometry3 {
                rotation: na30::UnitQuaternion::from_euler_angles(0.0, 0.0, yaw as f32),
                translation: na30::Translation3::new(x as f32, y as f32, 0.0),
            };

            let [ex, ey] = self.extends_2d();
            window.draw_rect((ex, ey), pose, color);
        }
    }
}
