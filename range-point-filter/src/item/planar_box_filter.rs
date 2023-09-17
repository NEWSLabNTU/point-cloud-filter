use anyhow::ensure;
use itertools::izip;
use nalgebra::{
    coordinates::{XY, XYZ},
    Isometry2, Isometry3, Point2, Point3, Translation2, Translation3, UnitComplex, UnitQuaternion,
};
use noisy_float::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanarBoxFilterConfig {
    pub xy: [R64; 2],
    pub wh: [R64; 2],
    pub z_min: Option<R64>,
    pub z_max: Option<R64>,
    pub azimuth_degrees: R64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "PlanarBoxFilterConfig", into = "PlanarBoxFilterConfig")]
pub struct PlanarBoxFilter {
    pub(crate) tf_2d: Isometry2<f64>,
    pub(crate) tf_2d_inv: Isometry2<f64>,
    pub(crate) ranges: [RangeInclusive<f64>; 3],
}

impl PlanarBoxFilter {
    pub fn contains(&self, point: &Point3<f64>) -> bool {
        let XYZ { x, y, z } = **point;
        let point_2d = self.tf_2d_inv * Point2::new(x, y);
        let XY { x, y } = *point_2d;
        izip!([x, y, z], &self.ranges).all(|(v, range)| range.contains(&v))
    }

    pub fn pose_3d(&self) -> Option<Isometry3<f64>> {
        let [x, y, z] = self.center_3d()?;
        let yaw = self.tf_2d.rotation.angle();

        let trans = Translation3::new(x, y, z);
        let rot = UnitQuaternion::from_euler_angles(0.0, 0.0, yaw);
        Some(Isometry3::from_parts(trans, rot))
    }

    pub fn pose_2d(&self) -> &Isometry2<f64> {
        &self.tf_2d
    }

    pub fn center_3d(&self) -> Option<[f64; 3]> {
        let XY { x, y } = *self.tf_2d.translation.vector;
        let z = self.z_center()?;
        Some([x, y, z])
    }

    pub fn center_2d(&self) -> [f64; 2] {
        let XY { x, y } = *self.tf_2d.translation.vector;
        [x, y]
    }

    pub fn extends_3d(&self) -> Option<[f64; 3]> {
        let len = |r: &RangeInclusive<f64>| (r.end() - r.start());

        let [rx, ry, rz] = &self.ranges;
        let ok = rz.start().is_finite() && rz.end().is_finite();
        ok.then(|| [len(rx), len(ry), len(rz)])
    }

    pub fn extends_2d(&self) -> [f64; 2] {
        let len = |r: &RangeInclusive<f64>| (r.end() - r.start());
        let [rx, ry, _] = &self.ranges;
        [len(rx), len(ry)]
    }

    fn z_center(&self) -> Option<f64> {
        let [_, _, z_range] = &self.ranges;
        let start = z_range.start();
        let end = z_range.end();
        let ok = start.is_finite() && end.is_finite();
        ok.then(|| start + (end - start) / 2.0)
    }
}

impl TryFrom<PlanarBoxFilterConfig> for PlanarBoxFilter {
    type Error = anyhow::Error;

    fn try_from(config: PlanarBoxFilterConfig) -> Result<Self, Self::Error> {
        let PlanarBoxFilterConfig {
            azimuth_degrees,
            xy: [x, y],
            wh,
            z_min,
            z_max,
        } = config;
        let yaw = azimuth_degrees.raw().to_radians();

        let z_min = match z_min {
            Some(val) => val.raw(),
            None => f64::neg_infinity(),
        };
        let z_max = match z_max {
            Some(val) => val.raw(),
            None => f64::infinity(),
        };

        let ranges = {
            ensure!(z_min <= z_max, "[{z_min}, {z_max}] is not a valid bound");

            let [ex, ey] = wh;
            let hx = ex.raw() / 2.0;
            let hy = ey.raw() / 2.0;
            let x_range = (-hx)..=hx;
            let y_range = (-hy)..=hy;
            let z_range = z_min..=z_max;
            [x_range, y_range, z_range]
        };

        let tf_2d = Isometry2 {
            translation: Translation2::new(x.raw(), y.raw()),
            rotation: UnitComplex::from_angle(yaw),
        };
        let tf_2d_inv = tf_2d.inverse();

        Ok(Self {
            tf_2d,
            tf_2d_inv,
            ranges,
        })
    }
}

impl From<PlanarBoxFilter> for PlanarBoxFilterConfig {
    fn from(from: PlanarBoxFilter) -> Self {
        let len = |r: RangeInclusive<f64>| (r64(r.end() - r.start()));

        let xy = {
            let XY { x, y } = *from.tf_2d.translation;
            [r64(x), r64(y)]
        };
        let azimuth_degrees = r64(from.tf_2d.rotation.angle().to_degrees());
        let [rx, ry, rz] = from.ranges;

        let wh = [len(rx), len(ry)];
        let z_min = R64::try_new(*rz.start());
        let z_max = R64::try_new(*rz.end());

        Self {
            azimuth_degrees,
            xy,
            wh,
            z_min,
            z_max,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::item::planar_box_filter::PlanarBoxFilter;
    use na::Point3;
    use nalgebra as na;

    #[test]
    fn inclusive_box_filter_test() {
        // Case 1
        {
            let filter: PlanarBoxFilter = json5::from_str(
                r#"
{
    "xy": [1, 2],
    "wh": [5, 10],
    "azimuth_degrees": 0
}
"#,
            )
            .unwrap();

            assert!(filter.contains(&Point3::new(3.4, 2.0, 0.0)));
            assert!(!filter.contains(&Point3::new(3.6, 2.0, 0.0)));
        }

        // Case 2
        {
            let filter: PlanarBoxFilter = json5::from_str(
                r#"
{
    "xy": [1, 2],
    "wh": [5, 10],
     "z_min": 4,
    "azimuth_degrees": 0
}
"#,
            )
            .unwrap();

            assert!(filter.contains(&Point3::new(3.4, 2.0, 5.0)));
            assert!(!filter.contains(&Point3::new(3.4, 2.0, 0.0)));
        }

        // Case 3
        {
            let filter: PlanarBoxFilter = json5::from_str(
                r#"
{
    "xy": [1, 2],
    "wh": [5, 10],
     "z_min": 4,
     "z_max": 5,
    "azimuth_degrees": 0
}
"#,
            )
            .unwrap();

            assert!(filter.contains(&Point3::new(3.4, 2.0, 4.5)));
            assert!(!filter.contains(&Point3::new(3.4, 2.0, 0.0)));
            assert!(!filter.contains(&Point3::new(3.4, 2.0, 6.0)));
        }
    }
}
