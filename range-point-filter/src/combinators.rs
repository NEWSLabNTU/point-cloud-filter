use crate::filter::PointFilter;
use nalgebra as na;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct All {
    pub filters: Vec<PointFilter>,
}

impl All {
    pub fn contains(&self, point: &na::Point3<f64>, intensity: Option<f64>) -> bool {
        self.filters
            .iter()
            .all(|filter| filter.contains(point, intensity))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Any {
    pub filters: Vec<PointFilter>,
}

impl Any {
    pub fn contains(&self, point: &na::Point3<f64>, intensity: Option<f64>) -> bool {
        self.filters
            .iter()
            .any(|filter| filter.contains(point, intensity))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Not {
    pub filter: Box<PointFilter>,
}

impl Not {
    pub fn contains(&self, point: &na::Point3<f64>, intensity: Option<f64>) -> bool {
        !self.filter.contains(point, intensity)
    }
}
