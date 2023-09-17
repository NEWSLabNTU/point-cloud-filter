use crate::config::Config;
use anyhow::Result;
use background_point_filter::BackgroundPointFilter;
use na::Point3;
use nalgebra as na;
use r2r::sensor_msgs::msg::PointCloud2;
use r2r_msg_ext::sensor_msgs::msg::{PointCloud2Ext, PointCloud2NalgebraExt};
use range_point_filter::FilterProgram as PointFilter;
use std::ops::{Bound, RangeBounds};

pub struct Filter {
    lidar_tf: na::Isometry3<f32>,
    ground_tf: na::Isometry3<f32>,
    ground_filter_range: Option<f32>,
    lidar_distance_bound: Option<(Bound<f32>, Bound<f32>)>,
    background_point_filter: Option<BackgroundPointFilter>,
    range_filter_rules: Option<PointFilter>,
}

impl Filter {
    pub fn new(
        config: &Config,
        lidar_tf: na::Isometry3<f32>,
        ground_tf: na::Isometry3<f32>,
    ) -> Self {
        let Config {
            ground_filter_range,
            lidar_distance_bound,
            ref background_point_filter,
            ref range_filter_rules,
            ..
        } = *config;

        let background_point_filter = background_point_filter
            .as_ref()
            .map(BackgroundPointFilter::new);

        Self {
            lidar_tf,
            ground_tf,
            ground_filter_range,
            lidar_distance_bound,
            background_point_filter,
            range_filter_rules: range_filter_rules.clone(),
        }
    }

    pub fn process_msg(&self, msg: &PointCloud2) -> Result<PointCloud2> {
        type BoxIter<'a> = Box<dyn Iterator<Item = (usize, Point3<f32>)> + Send + 'a>;

        let iter = msg.na_point_iter()?.enumerate();

        // Filter points by the distance to the lidar.
        let iter: BoxIter = match self.lidar_distance_bound {
            Some(range) => {
                // Run the filter if  lidar_distance_bound is set.
                let iter = iter.filter(move |(_index, world_pt)| {
                    // Convert the point to lidar-centered coordinate.
                    let lidar_pt = self.lidar_tf.inverse() * world_pt;
                    let distance = lidar_pt.coords.norm();
                    range.contains(&distance)
                });
                Box::new(iter)
            }
            None => Box::new(iter),
        };

        // Ground filter
        let iter: BoxIter = match self.ground_filter_range {
            Some(thresh) => {
                let iter = iter.filter(move |(_index, world_pt)| {
                    // Convert the point to ground-centered coordinate.
                    let ground_pt = self.ground_tf.inverse() * world_pt;
                    let distance = na::Point2::new(ground_pt.x, ground_pt.y).coords.norm();
                    distance >= thresh || ground_pt.z >= 0.0
                });
                Box::new(iter)
            }
            None => iter,
        };

        // Apply range filter
        let iter = match &self.range_filter_rules {
            Some(filter) => Box::new(iter.filter(|(_index, world_pt)| {
                let world_pt: na::Point3<f64> = na::convert_ref(world_pt);
                filter.contains(&world_pt, None)
            })),
            None => iter,
        };

        // Apply statistical background point filter
        let point_indices: Vec<usize> = match &self.background_point_filter {
            Some(filter) => {
                let points: Vec<_> = iter
                    .filter(|(_index, point)| {
                        let point: na::Point3<f64> = na::convert_ref(point);
                        filter.check_is_background(&point)
                    })
                    .map(|(index, _)| index)
                    .collect();
                filter.step();
                points
            }
            None => iter.map(|(index, _)| index).collect(),
        };

        let n_out_points = point_indices.len();
        let point_bytes: Vec<&[u8]> = msg.point_bytes_iter().flatten().collect();
        let out_data: Vec<u8> = point_indices
            .into_iter()
            .flat_map(|idx| point_bytes[idx])
            .cloned()
            .collect();
        let out_msg = PointCloud2 {
            header: msg.header.clone(),
            height: n_out_points as u32,
            width: 1,
            fields: msg.fields.clone(),
            is_bigendian: msg.is_bigendian,
            point_step: msg.point_step,
            row_step: msg.point_step,
            data: out_data,
            is_dense: msg.is_dense,
        };

        Ok(out_msg)
    }
}
