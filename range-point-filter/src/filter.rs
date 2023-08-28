use crate::{
    combinators::{All, Any, Not},
    inclusive_box_filter::PlanarBoxFilter,
    intensity_filter::IntensityFilter,
};
use nalgebra as na;
use serde::{Deserialize, Serialize};

/// The generic point cloud filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PointFilter {
    PlanarBox(PlanarBoxFilter),
    Intensity(IntensityFilter),
    All(All),
    Any(Any),
    Not(Not),
    True,
    False,
}

impl PointFilter {
    pub fn contains(&self, point: &na::Point3<f64>, intensity: Option<f64>) -> bool {
        match self {
            Self::PlanarBox(filter) => filter.contains(point),
            Self::Intensity(filter) => filter.contains(intensity),
            Self::All(filter) => filter.contains(point, intensity),
            Self::Any(filter) => filter.contains(point, intensity),
            Self::Not(filter) => filter.contains(point, intensity),
            Self::True => true,
            Self::False => false,
        }
    }
}

impl From<IntensityFilter> for PointFilter {
    fn from(v: IntensityFilter) -> Self {
        Self::Intensity(v)
    }
}

impl From<PlanarBoxFilter> for PointFilter {
    fn from(v: PlanarBoxFilter) -> Self {
        Self::PlanarBox(v)
    }
}

impl From<All> for PointFilter {
    fn from(v: All) -> Self {
        Self::All(v)
    }
}

impl From<Any> for PointFilter {
    fn from(v: Any) -> Self {
        Self::Any(v)
    }
}

impl From<Not> for PointFilter {
    fn from(v: Not) -> Self {
        Self::Not(v)
    }
}
