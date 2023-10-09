use crate::config::{self, Config};
use anyhow::Result;
use nalgebra::{coordinates::XYZ, Isometry3, Point3, Vector2};
use range_point_filter::FilterProgram;
use static_point_filter::StaticPointFilter;
use std::ops::{Bound, RangeBounds, RangeInclusive};

#[derive(Debug)]
pub struct Filter {
    ground_filter: Option<GroundFilter>,
    lidar_filter: Option<LidarFilter>,
    range_filter: Option<FilterProgram>,
    background_filter: Option<StaticPointFilter>,
}

impl Filter {
    pub fn new(config: &Config) -> Self {
        let Config {
            ground_filter,
            lidar_filter,
            range_filter,
            background_filter,
        } = config;
        let lidar_filter = lidar_filter.as_ref().map(LidarFilter::new);
        let ground_filter = ground_filter.as_ref().map(GroundFilter::new);
        Self {
            ground_filter,
            lidar_filter,
            range_filter: range_filter.clone(),
            background_filter: background_filter.clone(),
        }
    }

    pub fn process_msg(&self, points: Vec<Point3<f32>>) -> Result<Vec<Point3<f32>>> {
        type BoxIter<'a> = Box<dyn Iterator<Item = Point3<f32>> + Send + 'a>;

        let iter: BoxIter<'_> = Box::new(points.into_iter());

        // Filter points by the distance to the lidar.
        let iter: BoxIter = if let Some(filter) = &self.lidar_filter {
            let iter = iter.filter(move |pt| filter.contains(pt));
            Box::new(iter)
        } else {
            iter
        };

        // Ground filter
        let iter: BoxIter = if let Some(filter) = &self.ground_filter {
            let iter = iter.filter(move |pt| filter.contains(pt));
            Box::new(iter)
        } else {
            iter
        };

        // Apply range filter
        let iter = if let Some(filter) = &self.range_filter {
            let iter = iter.filter(|pt| {
                let pt: Point3<f64> = nalgebra::convert_ref(pt);
                filter.contains(&pt, None)
            });
            Box::new(iter)
        } else {
            iter
        };

        // Apply statistical background point filter
        let output_points: Vec<_> = if let Some(filter) = &self.background_filter {
            let points: Vec<_> = iter
                .filter(|pt| {
                    let pt: Point3<f64> = nalgebra::convert_ref(pt);
                    !filter.check_is_background(&pt)
                })
                .collect();

            filter.step();
            points
        } else {
            iter.collect()
        };

        Ok(output_points)
    }
}

#[derive(Debug)]
struct LidarFilter {
    transform: Isometry3<f32>,
    range: (Bound<f32>, Bound<f32>),
}

impl LidarFilter {
    pub fn new(config: &config::LidarFilter) -> Self {
        let config::LidarFilter {
            ref transform,
            min_distance,
            max_distance,
        } = *config;
        let lower = Bound::Included(min_distance.raw());
        let upper = match max_distance {
            Some(dist) => Bound::Included(dist.raw()),
            None => Bound::Unbounded,
        };

        LidarFilter {
            transform: transform.clone(),
            range: (lower, upper),
        }
    }

    pub fn contains(&self, point: &Point3<f32>) -> bool {
        let point = self.transform.inverse() * point;
        let XYZ { x, y, .. } = *point;
        let distance = Vector2::new(x, y).norm();
        self.range.contains(&distance)
    }
}

#[derive(Debug)]
struct GroundFilter {
    transform: Isometry3<f32>,
    range: RangeInclusive<f32>,
}

impl GroundFilter {
    pub fn new(config: &config::GroundFilter) -> Self {
        let config::GroundFilter {
            ref transform,
            max_distance,
        } = *config;

        GroundFilter {
            transform: transform.clone(),
            range: 0.0..=max_distance.raw(),
        }
    }

    pub fn contains(&self, point: &Point3<f32>) -> bool {
        let point = self.transform.inverse() * point;
        let XYZ { x, y, z } = *point;
        let distance = Vector2::new(x, y).norm();
        z >= 0.0 && self.range.contains(&distance)
    }
}
