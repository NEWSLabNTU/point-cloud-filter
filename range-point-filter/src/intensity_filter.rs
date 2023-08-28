use noisy_float::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntensityFilterConfig {
    pub min: R64,
    pub max: Option<R64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "IntensityFilterConfig", into = "IntensityFilterConfig")]
pub struct IntensityFilter {
    range: Range<N64>,
}

impl IntensityFilter {
    pub fn contains(&self, intensity: Option<f64>) -> bool {
        // TODO: change intensity to f64 type
        // true if intensity is not defined
        let intensity = if let Some(intensity) = intensity {
            intensity
        } else {
            return true;
        };

        // reject NaN
        let intensity = if let Some(intensity) = N64::try_new(intensity) {
            intensity
        } else {
            return false;
        };

        self.range.contains(&intensity)
    }
}

impl From<IntensityFilterConfig> for IntensityFilter {
    fn from(config: IntensityFilterConfig) -> Self {
        let min = config.min.raw();
        let max = config.max.map(|max| max.raw()).unwrap_or(f64::INFINITY);

        let min = n64(min);
        let max = n64(max);

        let range = min..max;
        Self { range }
    }
}

impl From<IntensityFilter> for IntensityFilterConfig {
    fn from(config: IntensityFilter) -> Self {
        let min = r64(config.range.start.raw());
        let max = {
            let max = config.range.end;
            max.is_finite().then(|| r64(max.raw()))
        };
        Self { min, max }
    }
}
