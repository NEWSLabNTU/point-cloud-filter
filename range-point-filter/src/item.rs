mod intensity_filter;
mod planar_box_filter;

use serde::{Deserialize, Serialize};

pub use self::{intensity_filter::IntensityFilter, planar_box_filter::PlanarBoxFilter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Item {
    Box(PlanarBoxFilter),
    Intensity(IntensityFilter),
}
