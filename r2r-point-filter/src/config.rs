use serde::{Deserialize, Serialize};
use std::ops::Bound;



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde_semver::SemverReq)]
#[version("0.3.0")]
pub struct Version;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: Version,
    pub enable_gui: bool,
    pub reliable: bool,
    pub ground_filter_range: Option<f32>,
    #[serde(with = "crate::utils::serde_option_bound", default)]
    pub lidar_distance_bound: Option<(Bound<f32>, Bound<f32>)>,
    pub background_point_filter: Option<background_point_filter::Config>,
    pub range_filter_rules: Option<range_point_filter::PointFilter>,
}

