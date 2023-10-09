use crate::config::{self, Config};
use anyhow::Result;
use nalgebra::{coordinates::XYZ, Isometry3, Point3, Scalar, Vector2};
use num::Float;
use range_point_filter::FilterProgram;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use static_point_filter::StaticPointFilter;
use std::ops::{Bound, RangeBounds, RangeInclusive};

pub type Pt32 = Pt<f32>;
pub type Pt64 = Pt<f64>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pt<T> {
    pub xyz: [T; 3],
    pub intensity: Option<T>,
}

impl<T> From<Point3<T>> for Pt<T>
where
    T: Scalar,
{
    fn from(src: Point3<T>) -> Self {
        Self {
            xyz: src.into(),
            intensity: None,
        }
    }
}

impl<T> From<&Point3<T>> for Pt<T>
where
    T: Scalar,
{
    fn from(src: &Point3<T>) -> Self {
        Self {
            xyz: (*src).clone().into(),
            intensity: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Filter {
    ground_filter: Option<GroundFilter>,
    lidar_filter: Option<LidarFilter>,
    range_filter: Option<FilterProgram>,
    background_filter: Option<StaticPointFilter>,
    config: Config,
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
            config: config.clone(),
        }
    }

    pub fn contains<T>(&self, point: &Pt<T>) -> bool
    where
        T: Float,
    {
        macro_rules! ensure {
            ($cond:expr) => {
                if !$cond {
                    return false;
                }
            };
        }

        let Pt {
            xyz: [x, y, z],
            intensity,
        } = *point;
        let p32: Point3<f32> = [
            num::cast(x).unwrap(),
            num::cast(y).unwrap(),
            num::cast(z).unwrap(),
        ]
        .into();
        let p64: Point3<f64> = [
            num::cast(x).unwrap(),
            num::cast(y).unwrap(),
            num::cast(z).unwrap(),
        ]
        .into();

        // Filter points by the distance to the lidar.
        if let Some(filter) = &self.lidar_filter {
            ensure!(filter.contains(&p32));
        }

        // Ground filter
        if let Some(filter) = &self.ground_filter {
            ensure!(filter.contains(&p32));
        }

        // Apply range filter
        if let Some(filter) = &self.range_filter {
            ensure!(filter.contains(&p64, intensity.map(|v| num::cast(v).unwrap())));
        }

        // Apply statistical background point filter
        if let Some(filter) = &self.background_filter {
            ensure!(!filter.check_is_background(&p64));
        }

        true
    }

    pub fn step(&self) {
        // Update background point filter statistics
        if let Some(filter) = &self.background_filter {
            filter.step();
        }
    }

    pub fn filter_frame<T, I>(&self, frame: I) -> Result<Vec<Pt<T>>>
    where
        T: Float,
        I: IntoIterator<Item = Pt<T>>,
    {
        let points: Vec<_> = frame.into_iter().filter(|p| self.contains(p)).collect();
        self.step();
        Ok(points)
    }
}

impl Serialize for Filter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.config.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Filter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let config = Config::deserialize(deserializer)?;
        let filter = Self::new(&config);
        Ok(filter)
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
