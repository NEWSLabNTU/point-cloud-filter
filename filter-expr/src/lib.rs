mod item;
mod parser;
mod serialization;
mod utils;

use self::serialization::{FilterExpr, SerializedFilterRule};
use anyhow::bail;
use item::Item;
use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "SerializedFilterRule", into = "SerializedFilterRule")]
pub struct FilterRule {
    expr: FilterExpr,
    items: HashMap<String, Item>,
}

impl TryFrom<SerializedFilterRule> for FilterRule {
    type Error = anyhow::Error;

    fn try_from(from: SerializedFilterRule) -> Result<Self, Self::Error> {
        let expr = &from.expr.0 .0;

        if let Err(ident) = from.check_expr(expr) {
            bail!(r#"The item {ident}" is not defined"#);
        }

        let SerializedFilterRule { expr, items } = from;
        Ok(Self { expr, items })
    }
}

impl From<FilterRule> for SerializedFilterRule {
    fn from(from: FilterRule) -> Self {
        let FilterRule { expr, items } = from;
        Self { expr, items }
    }
}

impl FilterRule {
    pub fn contains(&self, point: &Point3<f64>, intensity: Option<f64>) -> bool {
        todo!();
    }
}

struct Point<'a> {
    point: &'a Point3<f64>,
    intensity: Option<f64>,
}
