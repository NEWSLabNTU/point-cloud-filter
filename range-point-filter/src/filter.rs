use crate::item::Item;
use anyhow::bail;
use filter_expr::{
    normal_form::{Dnf, Term},
    parser::{Expr, ExprBinOp, Ident, Program},
};
use nalgebra_0_32::Point3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "SerializedFilterProgram", into = "SerializedFilterProgram")]
pub struct FilterProgram {
    pub(crate) dnf: Dnf,
    pub(crate) program: Program,
    pub(crate) items: HashMap<String, Item>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializedFilterProgram {
    program: Program,
    items: HashMap<String, Item>,
}

impl SerializedFilterProgram {
    fn check_program(&self, program: &Program) -> Result<(), String> {
        self.check_expr(&program.0)
    }

    fn check_expr(&self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Ident(ident) => self.check_ident(ident),
            Expr::UnaryOp(expr) => self.check_expr(expr),
            Expr::BinOp(ExprBinOp { lhs, rhs, .. }) => {
                self.check_expr(lhs)?;
                self.check_expr(rhs)?;
                Ok(())
            }
        }
    }

    fn check_ident(&self, ident: &Ident) -> Result<(), String> {
        if self.items.contains_key(&ident.0) {
            Ok(())
        } else {
            Err(ident.to_string())
        }
    }
}

impl TryFrom<SerializedFilterProgram> for FilterProgram {
    type Error = anyhow::Error;

    fn try_from(from: SerializedFilterProgram) -> Result<Self, Self::Error> {
        if let Err(ident) = from.check_program(&from.program) {
            bail!(r#"The item {ident}" is not defined"#);
        }

        let dnf = Dnf::from_program(from.program.clone());

        Ok(Self {
            dnf,
            items: from.items,
            program: from.program,
        })
    }
}

impl From<FilterProgram> for SerializedFilterProgram {
    fn from(from: FilterProgram) -> Self {
        let FilterProgram {
            items,
            program: cached_lang,
            ..
        } = from;

        Self {
            items,
            program: cached_lang,
        }
    }
}

impl FilterProgram {
    pub fn contains(&self, point: &Point3<f64>, intensity: Option<f64>) -> bool {
        self.dnf.0.iter().any(|product| {
            product.0.iter().all(|term| {
                let Term { ref ident, invert } = *term;
                let item = &self.items[&ident.0];

                let yes = match item {
                    Item::Box(filter) => filter.contains(point),
                    Item::Intensity(filter) => filter.contains(intensity),
                };

                invert ^ yes
            })
        })
    }
}
