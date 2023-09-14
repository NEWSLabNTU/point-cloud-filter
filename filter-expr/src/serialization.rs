use super::parser::{parse_filter_str, Expr, Ident, Lang, SubTerm, Term};
use crate::item::Item;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedFilterRule {
    pub expr: FilterExpr,
    pub items: HashMap<String, Item>,
}

impl SerializedFilterRule {
    pub(crate) fn check_expr(&self, expr: &Expr) -> Result<(), String> {
        let Expr {
            lhs_term,
            rhs_terms,
        } = expr;

        self.check_term(lhs_term)?;
        rhs_terms
            .iter()
            .try_for_each(|(_, term)| self.check_term(term))
    }

    pub(crate) fn check_term(&self, term: &Term) -> Result<(), String> {
        self.check_subterm(&term.subterm)
    }

    pub(crate) fn check_subterm(&self, subterm: &SubTerm) -> Result<(), String> {
        match subterm {
            SubTerm::Ident(ident) => self.check_ident(ident),
            SubTerm::Expr(expr) => self.check_expr(expr),
        }
    }

    pub(crate) fn check_ident(&self, ident: &Ident) -> Result<(), String> {
        if self.items.contains_key(&ident.0) {
            Ok(())
        } else {
            Err(ident.0.to_string())
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilterExpr(pub(crate) Lang);

impl Serialize for FilterExpr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let text = format!("{}", self.0);
        text.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FilterExpr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        let lang = parse_filter_str(&text).map_err(|err| D::Error::custom(format!("{err}")))?;
        Ok(Self(lang))
    }
}
