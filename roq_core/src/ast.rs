use serde::{Deserialize, Serialize};

// TODO: Newtype wrapper restricting this to valid Coq identifiers.
pub type Ident = String;

/// A Coq vernacular file.
pub struct Vernacular {
    pub statements: Vec<Statement>,
}

impl From<Statement> for Vernacular {
    fn from(stmt: Statement) -> Self {
        Vernacular {
            statements: vec![stmt],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Statement {
    Definition(Definition),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Definition {
    pub name: Ident,
    pub args: Vec<Binder>,
    pub ret: Ty,
    pub body: Expr,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Binder {
    pub name: Ident,
    pub ty: Ty,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Ty {
    Nat,
    Bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Expr {
    /// Apply a function.
    Apply { func: String, args: Vec<Expr> },

    /// A variable name.
    Var(Ident),

    /// A `nat` literal.
    Nat(u64),

    /// A `bool` literal.
    Bool(bool),
}
