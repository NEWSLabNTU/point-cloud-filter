use crate::{
    item::{Item, PlanarBoxFilter},
    FilterProgram,
};
use filter_expr::normal_form::Term;
use kiss3d::window::Window;
use kiss3d_utils::WindowPlotExt as _;
use nalgebra::{coordinates::XY, Isometry2, Isometry3, Point3, Translation3, UnitQuaternion};

impl FilterProgram {
    pub fn render_kiss3d(&self, window: &mut Window) {
        for product in &self.dnf.0 {
            for term in &product.0 {
                let Term { ref ident, invert } = *term;

                let pos_color = Point3::new(0.0, 1.0, 0.0);
                let neg_color = Point3::new(1.0, 0.0, 0.0);
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
    pub fn render_kiss3d(&self, window: &mut Window, color: &Point3<f32>) {
        if let Some(pose) = self.pose_3d() {
            let pose: Isometry3<f32> = nalgebra::convert(pose);
            let extends = self.extends_3d().unwrap();
            window.draw_box(extends, pose, color);
        } else {
            let Isometry2 {
                rotation,
                translation,
            } = self.pose_2d();

            let XY { x, y } = *translation.vector;
            let yaw = rotation.angle();

            let pose = Isometry3 {
                rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, yaw as f32),
                translation: Translation3::new(x as f32, y as f32, 0.0),
            };

            let [ex, ey] = self.extends_2d();
            window.draw_rect((ex, ey), pose, color);
        }
    }
}
