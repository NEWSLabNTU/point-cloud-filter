use crate::{Not, PlanarBoxFilter, PointFilter};
use kiss3d::window::Window;
use kiss3d_utils::WindowPlotExt as _;
use nalgebra as na;
use std::ops::{Bound::*, RangeBounds};

impl PointFilter {
    pub fn render_kiss3d(&self, window: &mut Window) {
        render_recursive(self, window, true);
    }
}

fn render_recursive(filter: &PointFilter, window: &mut Window, is_positive: bool) {
    use PointFilter as F;

    let pos_color = na::Point3::new(0.0, 1.0, 0.0);
    let neg_color = na::Point3::new(1.0, 0.0, 0.0);

    let major_color = if is_positive { &pos_color } else { &neg_color };

    match filter {
        F::PlanarBox(pbox) => {
            draw_planar_box(window, pbox, major_color);
        }
        F::Intensity(_) => {}
        F::All(all) => {
            all.filters.iter().for_each(|filter| {
                render_recursive(filter, window, is_positive);
            });
        }
        F::Any(any) => {
            any.filters.iter().for_each(|filter| {
                render_recursive(filter, window, is_positive);
            });
        }
        F::Not(Not { filter }) => {
            render_recursive(filter, window, !is_positive);
        }
        F::True => {}
        F::False => {}
    }
}

fn draw_planar_box(window: &mut Window, filter: &PlanarBoxFilter, color: &na::Point3<f32>) {
    let pose: na::Isometry3<f32> = na::convert_ref(filter.pose());
    let size_x = filter.size_x();
    let size_y = filter.size_y();
    let z_bound = filter.z_bound();

    match (z_bound.start_bound(), z_bound.end_bound()) {
        (Unbounded, Unbounded) => {}
        (Included(_) | Excluded(_), Unbounded) | (Unbounded, Included(_) | Excluded(_)) => {
            window.draw_rect((size_x, size_y), pose, color);
        }
        (Included(z_min) | Excluded(z_min), Included(z_max) | Excluded(z_max)) => {
            let size = na::Point3::new(size_x, size_y, (*z_max - *z_min).raw());
            window.draw_box(size, pose, color);
        }
    }
}
