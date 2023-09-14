use nalgebra as na;
use noisy_float::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Bound, RangeBounds as _};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanarBoxFilterConfig {
    #[serde(with = "crate::utils::serde_bound")]
    pub z_bound: (Bound<R64>, Bound<R64>),
    pub size_x: R64,
    pub size_y: R64,
    pub center_x: R64,
    pub center_y: R64,
    pub azimuth_degrees: R64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "PlanarBoxFilterConfig", into = "PlanarBoxFilterConfig")]
pub struct PlanarBoxFilter {
    pose: na::Isometry3<f64>,
    inverse_pose: na::Isometry3<f64>,
    size_x: f64,
    size_y: f64,
    #[serde(with = "crate::utils::serde_bound")]
    pub z_bound: (Bound<R64>, Bound<R64>),
}

impl PlanarBoxFilter {
    pub fn size_x(&self) -> f64 {
        self.size_x
    }

    pub fn size_y(&self) -> f64 {
        self.size_y
    }

    pub fn pose(&self) -> &na::Isometry3<f64> {
        &self.pose
    }

    pub fn inverse_pose(&self) -> &na::Isometry3<f64> {
        &self.inverse_pose
    }

    pub fn z_bound(&self) -> &(Bound<R64>, Bound<R64>) {
        &self.z_bound
    }

    pub fn contains(&self, point: &na::Point3<f64>) -> bool {
        let point = self.inverse_pose * point;
        self.z_bound.contains(&r64(point.z))
            && point.x >= -self.size_x / 2.0
            && point.x <= self.size_x / 2.0
            && point.y >= -self.size_y / 2.0
            && point.y <= self.size_y / 2.0
    }
}

impl From<PlanarBoxFilterConfig> for PlanarBoxFilter {
    fn from(config: PlanarBoxFilterConfig) -> Self {
        let pose = {
            let trans = na::Translation3::new(config.center_x.raw(), config.center_y.raw(), 0.0);
            let rot = na::UnitQuaternion::from_euler_angles(
                0.0,
                0.0,
                config.azimuth_degrees.raw().to_radians(),
            );
            na::Isometry3::from_parts(trans, rot)
        };

        Self {
            pose,
            inverse_pose: pose.inverse(),
            size_x: config.size_x.raw(),
            size_y: config.size_y.raw(),
            z_bound: config.z_bound,
        }
    }
}

impl From<PlanarBoxFilter> for PlanarBoxFilterConfig {
    fn from(from: PlanarBoxFilter) -> Self {
        let PlanarBoxFilter {
            pose:
                na::Isometry3 {
                    rotation,
                    translation,
                },
            size_x,
            size_y,
            z_bound,
            ..
        } = from;

        Self {
            z_bound,
            size_x: r64(size_x),
            size_y: r64(size_y),
            center_x: r64(translation.x),
            center_y: r64(translation.y),
            azimuth_degrees: r64(rotation.euler_angles().2),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::item::planar_box_filter::{PlanarBoxFilter, PlanarBoxFilterConfig};
    use nalgebra as na;
    use noisy_float::prelude::*;
    use std::ops::Bound::*;

    #[test]
    fn inclusive_box_filter_test() {
        let filter: PlanarBoxFilter = PlanarBoxFilterConfig {
            z_bound: (Unbounded, Unbounded),
            size_x: r64(5.0),
            size_y: r64(10.0),
            center_x: r64(1.0),
            center_y: r64(2.0),
            azimuth_degrees: r64(0.0),
        }
        .into();
        assert!(filter.contains(&na::Point3::new(3.4, 2.0, 0.0)));
        assert!(!filter.contains(&na::Point3::new(3.6, 2.0, 0.0)));
    }
}
