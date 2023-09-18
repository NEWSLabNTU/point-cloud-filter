use noisy_float::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub range: Range,
    pub voxel_size: VoxelSize,
    pub background_threshold: R64,
}

impl Config {
    pub fn build(&self) -> super::StaticPointFilter {
        super::StaticPointFilter::new(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    #[serde(with = "crate::utils::serde_bound")]
    pub x_bound: RangeInclusive<R64>,
    #[serde(with = "crate::utils::serde_bound")]
    pub y_bound: RangeInclusive<R64>,
    #[serde(with = "crate::utils::serde_bound")]
    pub z_bound: RangeInclusive<R64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoxelSize {
    pub x_size: R64,
    pub y_size: R64,
    pub z_size: R64,
}
