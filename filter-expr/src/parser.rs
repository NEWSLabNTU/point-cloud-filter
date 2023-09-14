use anyhow::Result;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use std::fmt::{self, Display, Formatter};

#[derive(Parser)]
#[grammar = "rule.pest"]
pub struct ExprParser;

pub(crate) fn parse_filter_str(input: &str) -> Result<Lang> {
    let mut pairs = ExprParser::parse(Rule::lang, input)?;

    let lang = pairs.next().unwrap();
    let lang = parse_lang(lang);

    // end of iterator
    debug_assert!(pairs.next().is_none());

    Ok(lang)
}

fn parse_lang(pair: Pair<Rule>) -> Lang {
    let mut inner = pair.into_inner();
    let expr = inner.next().unwrap();
    let expr = parse_expr(expr);

    // Consume EOI
    let eoi = inner.next().unwrap();
    debug_assert_eq!(eoi.as_rule(), Rule::EOI);

    // end of iterator
    debug_assert!(inner.next().is_none());

    Lang(expr)
}

fn parse_expr(pair: Pair<Rule>) -> Expr {
    debug_assert_eq!(pair.as_rule(), Rule::expr);

    let mut inner = pair.into_inner();
    let lhs_term = inner.next().unwrap();
    let lhs_term = parse_term(lhs_term);

    let mut rhs_terms = vec![];

    while let Some(bin_op) = inner.next() {
        let rhs_term = inner.next().unwrap();

        let bin_op = parse_bin_op(bin_op);
        let rhs_term = parse_term(rhs_term);
        rhs_terms.push((bin_op, rhs_term));
    }

    Expr {
        lhs_term,
        rhs_terms,
    }
}

fn parse_term(pair: Pair<Rule>) -> Term {
    let mut inner = pair.into_inner();
    let pair = inner.next().unwrap();
    let term = match pair.as_rule() {
        Rule::subterm => {
            let subterm = parse_subterm(pair);
            Term {
                neg: false,
                subterm,
            }
        }
        Rule::unary_op => {
            let pair = inner.next().unwrap();
            let subterm = parse_subterm(pair);
            Term { neg: true, subterm }
        }
        _ => unreachable!(),
    };

    debug_assert!(inner.next().is_none());

    term
}

fn parse_subterm(pair: Pair<Rule>) -> SubTerm {
    debug_assert_eq!(pair.as_rule(), Rule::subterm);

    let mut inner = pair.into_inner();
    let pair = inner.next().unwrap();

    let subterm = match pair.as_rule() {
        Rule::ident => {
            let ident = parse_ident(pair);
            SubTerm::Ident(ident)
        }
        Rule::expr => {
            let expr = parse_expr(pair);
            SubTerm::Expr(Box::new(expr))
        }
        _ => unreachable!(),
    };

    debug_assert!(inner.next().is_none());
    subterm
}

fn parse_bin_op(pair: Pair<Rule>) -> BinOp {
    debug_assert_eq!(pair.as_rule(), Rule::bin_op);
    BinOp::from_str(pair.as_str())
}

fn parse_ident(pair: Pair<Rule>) -> Ident {
    debug_assert_eq!(pair.as_rule(), Rule::ident);
    Ident(pair.as_str().to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Lang(pub(crate) Expr);

impl Display for Lang {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self(expr) = self;
        write!(f, "{expr}")
    }
}

impl From<Expr> for Lang {
    fn from(expr: Expr) -> Self {
        Self(expr)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Expr {
    pub lhs_term: Term,
    pub rhs_terms: Vec<(BinOp, Term)>,
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            lhs_term,
            rhs_terms,
        } = self;

        write!(f, "{lhs_term}")?;

        for (op, term) in rhs_terms {
            write!(f, " {op} {term}")?;
        }

        Ok(())
    }
}

impl Expr {
    pub fn from_expr(expr: Expr) -> Self {
        Self {
            lhs_term: expr.into(),
            rhs_terms: vec![],
        }
    }
}

impl From<Term> for Expr {
    fn from(term: Term) -> Self {
        Self {
            lhs_term: term,
            rhs_terms: vec![],
        }
    }
}

impl From<SubTerm> for Expr {
    fn from(subterm: SubTerm) -> Self {
        Self {
            lhs_term: subterm.into(),
            rhs_terms: vec![],
        }
    }
}

impl From<Ident> for Expr {
    fn from(ident: Ident) -> Self {
        Self {
            lhs_term: ident.into(),
            rhs_terms: vec![],
        }
    }
}

impl From<&str> for Expr {
    fn from(v: &str) -> Self {
        Self {
            lhs_term: v.into(),
            rhs_terms: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Term {
    pub neg: bool,
    pub subterm: SubTerm,
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { neg, ref subterm } = *self;

        if neg {
            write!(f, "!")?;
        }

        write!(f, "{subterm}")
    }
}

impl From<Ident> for Term {
    fn from(ident: Ident) -> Self {
        Self {
            neg: false,
            subterm: ident.into(),
        }
    }
}

impl From<&str> for Term {
    fn from(v: &str) -> Self {
        Self {
            neg: false,
            subterm: v.into(),
        }
    }
}

impl From<Expr> for Term {
    fn from(expr: Expr) -> Self {
        Self {
            neg: false,
            subterm: expr.into(),
        }
    }
}

impl From<SubTerm> for Term {
    fn from(subterm: SubTerm) -> Self {
        Self {
            neg: false,
            subterm,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SubTerm {
    Ident(Ident),
    Expr(Box<Expr>),
}

impl Display for SubTerm {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SubTerm::Ident(ident) => write!(f, "{ident}"),
            SubTerm::Expr(expr) => write!(f, "({expr})"),
        }
    }
}

impl From<Expr> for SubTerm {
    fn from(v: Expr) -> Self {
        Self::Expr(Box::new(v))
    }
}

impl From<Ident> for SubTerm {
    fn from(v: Ident) -> Self {
        Self::Ident(v)
    }
}

impl From<&str> for SubTerm {
    fn from(v: &str) -> Self {
        Self::Ident(v.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BinOp {
    Mul,
    Add,
    Minus,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let op = match self {
            BinOp::Mul => "*",
            BinOp::Add => "+",
            BinOp::Minus => "-",
        };
        write!(f, "{op}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Ident(pub String);

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Ident(ident) = self;
        write!(f, "{ident}")
    }
}

impl Ident {
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<&str> for Ident {
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

impl BinOp {
    pub fn from_str(s: &str) -> Self {
        match s {
            "*" => BinOp::Mul,
            "+" => BinOp::Add,
            "-" => BinOp::Minus,
            _ => unreachable!("unexpected operator '{s}'"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Lang;
    use super::{BinOp, Expr, Term};
    use crate::parser::parse_filter_str;

    #[test]
    fn expr_parse() {
        let input = r#"a + !(b - c) * d"#;
        let lang = parse_filter_str(input).unwrap();

        let expect = Lang(Expr {
            lhs_term: "a".into(),
            rhs_terms: vec![
                (BinOp::Add, {
                    let inner = Expr {
                        lhs_term: "b".into(),
                        rhs_terms: vec![(BinOp::Minus, "c".into())],
                    };

                    Term {
                        neg: true,
                        subterm: inner.into(),
                    }
                }),
                (BinOp::Mul, "d".into()),
            ],
        });

        assert_eq!(lang, expect);
    }

    #[test]
    fn expr_to_string() {
        let input = "a + !(b - c) * d";
        let lang = parse_filter_str(input).unwrap();
        assert_eq!(format!("{lang}"), "a + !(b - c) * d");
    }
}
