use anyhow::Result;
use once_cell::sync::Lazy;
use pest::{
    iterators::Pair,
    pratt_parser::{Assoc, Op, PrattParser},
    Parser,
};
use pest_derive::Parser;
use std::fmt::{self, Display, Formatter};

static PARSER: Lazy<PrattParser<Rule>> = Lazy::new(|| -> PrattParser<Rule> {
    PrattParser::new()
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left))
        .op(Op::prefix(Rule::invert))
});

pub fn parse_str(input: &str) -> Result<Program> {
    let mut pairs = ExprParser::parse(Rule::program, input)?;
    let pair = pairs.next().unwrap();
    debug_assert!(pairs.next().is_none());
    Ok(Program::parse(pair))
}

#[derive(Parser)]
#[grammar = "rule.pest"]
pub struct ExprParser;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program(pub Expr);

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self(expr) = self;
        write!(f, "{expr}")
    }
}

impl Program {
    fn parse(pair: Pair<Rule>) -> Self {
        debug_assert_eq!(pair.as_rule(), Rule::program);

        let mut inner = pair.into_inner();
        let expr = Expr::parse(inner.next().unwrap());

        // Consume EOI
        let eoi = inner.next().unwrap();
        debug_assert_eq!(eoi.as_rule(), Rule::EOI);

        // end of iterator
        debug_assert!(inner.next().is_none());

        Self(expr)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Ident(Ident),
    UnaryOp(Box<Expr>),
    BinOp(ExprBinOp),
}

impl Expr {
    fn parse(pair: Pair<Rule>) -> Self {
        debug_assert_eq!(pair.as_rule(), Rule::expr);

        let inner = pair.into_inner();

        PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::ident => Expr::Ident(Ident::parse(primary)),
                Rule::expr => Expr::parse(primary),
                _ => unreachable!(),
            })
            .map_infix(|lhs, op, rhs| match op.as_rule() {
                Rule::add => ExprBinOp {
                    op: BinOp::Add,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
                .into(),
                Rule::sub => ExprBinOp {
                    op: BinOp::Mul,
                    lhs: Box::new(lhs),
                    rhs: Box::new(Expr::UnaryOp(Box::new(rhs))),
                }
                .into(),
                Rule::mul => ExprBinOp {
                    op: BinOp::Mul,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
                .into(),
                _ => unreachable!(),
            })
            .map_prefix(|_op, expr| Expr::UnaryOp(Box::new(expr)))
            .parse(inner)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Ident(ident) => write!(f, "{ident}"),
            Expr::UnaryOp(expr) => write!(f, "!({expr})"),
            Expr::BinOp(ExprBinOp { op, lhs, rhs }) => write!(f, "({lhs}) {op} ({rhs})"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExprBinOp {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

impl From<ExprBinOp> for Expr {
    fn from(value: ExprBinOp) -> Self {
        Self::BinOp(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    Mul,
    Add,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let op = match self {
            BinOp::Mul => "*",
            BinOp::Add => "+",
        };
        write!(f, "{op}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(pub String);

impl Ident {
    fn parse(pair: Pair<Rule>) -> Self {
        debug_assert_eq!(pair.as_rule(), Rule::ident);
        Self(pair.as_str().to_string())
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_test() {
        let input = "f - !a * !(b + c) * d + e";
        let program = super::parse_str(input).unwrap();

        assert_eq!(
            format!("{program}"),
            "((f) * (!(((!(a)) * (!((b) + (c)))) * (d)))) + (e)"
        );
    }
}
