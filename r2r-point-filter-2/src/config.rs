use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde_semver::SemverReq)]
#[version("0.3.0")]
pub struct Version;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: Version,
    pub enable_gui: bool,
    pub dataflow: String,
    pub infra: String,
    pub major_device: String,
    pub filter:FilterConfig
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig{
    pub ground_filter_range: f32,
    pub lidar_distance_bound: DistanceBound,
    pub rules: Rules,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceBound {
    pub imin: f32,
    pub imax: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules{
    pub r#type: String,
    pub filters:Vec<FilterRules>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRules{
    pub r#type: String,
    pub z_bound: DistanceBound,
    pub size_x: f32,
    pub size_y: f32,
    pub center_x: f32,
    pub center_y: f32,
    pub azimuth_degrees: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPath{
    pub filter_config:PathBuf,
    pub lidar_tf:PathBuf,
    pub ground_tf:PathBuf,
    pub data_path:PathBuf,
    pub output_path:PathBuf,
}
