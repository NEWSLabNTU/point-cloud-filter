use nalgebra::Isometry3;
use noisy_float::types::{N32, R32};
use range_point_filter::FilterProgram;
use serde::{Deserialize, Serialize};
use static_point_filter::StaticPointFilter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ground_filter: Option<GroundFilter>,
    pub lidar_filter: Option<LidarFilter>,
    pub range_filter: Option<FilterProgram>,
    pub background_filter: Option<StaticPointFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundFilter {
    /// The transform from world to ground coordinate.
    #[serde(with = "newslab_serde_nalgebra::isometry3_as_euler_angles")]
    pub transform: Isometry3<f32>,

    /// The maximum effective distance from the ground center.
    pub max_distance: N32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarFilter {
    /// The transform from world to lidar coordinate.
    #[serde(with = "newslab_serde_nalgebra::isometry3_as_euler_angles")]
    pub transform: Isometry3<f32>,

    /// The minimum effective distance from the LiDAR center.
    pub min_distance: R32,

    /// The maximum effective distance from the LiDAR center.
    pub max_distance: Option<R32>,
}
