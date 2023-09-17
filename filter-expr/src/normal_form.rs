use crate::parser::{BinOp, Expr, ExprBinOp, Ident, Program};
use itertools::{chain, Itertools};
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Clone)]
pub struct Cnf(pub Vec<DisjSum>);

impl Cnf {
    pub fn from_program(lang: Program) -> Self {
        Self::from_expr(lang.0)
    }

    fn from_expr(expr: Expr) -> Self {
        match expr {
            Expr::Ident(ident) => ident.into(),
            Expr::UnaryOp(expr) => Self::from_expr(*expr).invert(),
            Expr::BinOp(ExprBinOp { op, lhs, rhs }) => {
                let lhs = Self::from_expr(*lhs);
                let rhs = Self::from_expr(*rhs);

                match op {
                    BinOp::Mul => lhs.conj_with(rhs),
                    BinOp::Add => lhs.disj_with(rhs),
                }
            }
        }
        .reduce()
    }

    fn invert(self) -> Self {
        let products: Vec<_> = self.0.into_iter().map(|sum| sum.invert()).collect();
        let dnf = Dnf(products);
        dnf.into()
    }

    fn conj_with(self, other: Self) -> Self {
        Self(chain!(self.0, other.0).collect())
    }

    fn disj_with(self, other: Self) -> Self {
        let lhs: Dnf = self.into();
        let rhs: Dnf = other.into();
        lhs.disj_with(rhs).reduce().into()
    }

    fn reduce(self) -> Self {
        Self(self.0.into_iter().filter(|sum| !sum.is_true()).collect())
    }
}

impl Display for Cnf {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter();

        if let Some(sum) = iter.next() {
            write!(f, "({sum})")?;
        }

        for sum in iter {
            write!(f, " * ({sum})")?;
        }

        Ok(())
    }
}

impl From<Ident> for Cnf {
    fn from(ident: Ident) -> Self {
        Cnf(vec![ident.into()])
    }
}

impl From<Cnf> for Dnf {
    fn from(cnf: Cnf) -> Self {
        Self(
            cnf.0
                .into_iter()
                .map(|DisjSum(terms)| Vec::from_iter(terms))
                .multi_cartesian_product()
                .map(|terms| ConjProduct(HashSet::from_iter(terms)))
                .collect(),
        )
    }
}

impl From<Dnf> for Cnf {
    fn from(dnf: Dnf) -> Self {
        Self(
            dnf.0
                .into_iter()
                .map(|ConjProduct(terms)| Vec::from_iter(terms))
                .multi_cartesian_product()
                .map(|terms| DisjSum(HashSet::from_iter(terms)))
                .collect(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Dnf(pub Vec<ConjProduct>);

impl Dnf {
    pub fn from_program(lang: Program) -> Self {
        Self::from_expr(lang.0)
    }

    fn from_expr(expr: Expr) -> Self {
        match expr {
            Expr::Ident(ident) => ident.into(),
            Expr::UnaryOp(expr) => Self::from_expr(*expr).invert(),
            Expr::BinOp(ExprBinOp { op, lhs, rhs }) => {
                let lhs = Self::from_expr(*lhs);
                let rhs = Self::from_expr(*rhs);

                match op {
                    BinOp::Mul => lhs.conj_with(rhs),
                    BinOp::Add => lhs.disj_with(rhs),
                }
            }
        }
        .reduce()
    }

    fn conj_with(self, other: Self) -> Self {
        let lhs: Cnf = self.into();
        let rhs: Cnf = other.into();
        lhs.conj_with(rhs).reduce().into()
    }

    fn disj_with(self, other: Self) -> Self {
        Self(chain!(self.0, other.0).collect())
    }

    fn reduce(self) -> Self {
        Self(
            self.0
                .into_iter()
                .filter(|product| !product.is_false())
                .collect(),
        )
    }

    fn invert(self) -> Self {
        let products: Vec<_> = self.0.into_iter().map(|product| product.invert()).collect();
        let cnf = Cnf(products);
        cnf.into()
    }
}

impl From<Term> for Dnf {
    fn from(term: Term) -> Self {
        Self(vec![term.into()])
    }
}

impl From<Ident> for Dnf {
    fn from(ident: Ident) -> Self {
        Term::from(ident).into()
    }
}

impl Display for Dnf {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter();

        if let Some(sum) = iter.next() {
            write!(f, "{sum}")?;
        }

        for sum in iter {
            write!(f, " + {sum}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ConjProduct(pub HashSet<Term>);

impl ConjProduct {
    fn invert(self) -> DisjSum {
        let Self(terms) = self;
        let terms = terms.into_iter().map(|term| term.invert()).collect();
        DisjSum(terms)
    }

    fn is_false(&self) -> bool {
        #[derive(Default)]
        struct Entry {
            pub pos: bool,
            pub neg: bool,
        }

        let mut idents: HashMap<&Ident, Entry> = HashMap::new();

        for term in &self.0 {
            let Term { invert, ref ident } = *term;
            let entry = idents.entry(ident).or_insert_with(|| Entry::default());
            if invert {
                entry.neg = false;
            } else {
                entry.pos = true;
            }

            if entry.pos && entry.neg {
                return true;
            }
        }

        false
    }
}

impl From<Term> for ConjProduct {
    fn from(term: Term) -> Self {
        Self([term].into_iter().collect())
    }
}

impl Display for ConjProduct {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // for term in &self.0 {}
        let mut terms: Vec<_> = self.0.iter().collect();
        terms.sort_unstable();

        let mut iter = terms.into_iter();
        if let Some(term) = iter.next() {
            write!(f, "{term}")?;
        }

        for term in iter {
            write!(f, " * {term}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DisjSum(pub HashSet<Term>);

impl DisjSum {
    fn invert(self) -> ConjProduct {
        let Self(terms) = self;
        let terms = terms.into_iter().map(|term| term.invert()).collect();
        ConjProduct(terms)
    }

    fn is_true(&self) -> bool {
        #[derive(Default)]
        struct Entry {
            pub pos: bool,
            pub neg: bool,
        }

        let mut idents: HashMap<&Ident, Entry> = HashMap::new();

        for term in &self.0 {
            let Term { invert, ref ident } = *term;
            let entry = idents.entry(ident).or_insert_with(|| Entry::default());
            if invert {
                entry.neg = false;
            } else {
                entry.pos = true;
            }

            if entry.pos && entry.neg {
                return true;
            }
        }

        false
    }
}

impl From<Ident> for DisjSum {
    fn from(ident: Ident) -> Self {
        Term::from(ident).into()
    }
}

impl From<Term> for DisjSum {
    fn from(term: Term) -> Self {
        Self([term].into_iter().collect())
    }
}

impl Display for DisjSum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut terms: Vec<_> = self.0.iter().collect();
        terms.sort_unstable();

        let mut iter = terms.into_iter();
        if let Some(term) = iter.next() {
            write!(f, "{term}")?;
        }

        for term in iter {
            write!(f, " + {term}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Term {
    pub ident: Ident,
    pub invert: bool,
}

impl Term {
    fn invert(self) -> Self {
        let Self {
            invert: negate,
            ident,
        } = self;

        Self {
            invert: !negate,
            ident,
        }
    }
}

impl From<Ident> for Term {
    fn from(ident: Ident) -> Self {
        Self {
            ident,
            invert: false,
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { invert, ref ident } = *self;
        let op = if invert { "!" } else { "" };
        write!(f, "{op}{ident}")
    }
}
