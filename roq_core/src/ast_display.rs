use crate::ast;
use std::fmt;

impl fmt::Display for ast::Vernacular {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for stmt in &self.statements {
            writeln!(f, "{}", stmt)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl fmt::Display for ast::Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ast::Statement::Definition(defn) => write!(f, "{}", defn),
        }
    }
}

impl fmt::Display for ast::Definition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Definition {}", self.name)?;
        for binder in &self.args {
            write!(f, " ({binder})")?;
        }
        write!(f, " : {}", self.ret)?;
        writeln!(f, " :=")?;
        write!(f, "{}", Indent::tab(&self.body))?;
        writeln!(f, ".")?;
        Ok(())
    }
}

impl fmt::Display for ast::Binder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl fmt::Display for ast::Ty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ast::Ty::Nat => write!(f, "nat"),
            ast::Ty::Bool => write!(f, "bool"),
        }
    }
}

impl fmt::Display for ast::Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ast::Expr::Apply { func, args } => {
                write!(f, "({}", func)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                writeln!(f, ")")?;
                Ok(())
            }
            ast::Expr::Var(ident) => write!(f, "{ident}"),
            ast::Expr::Nat(n) => write!(f, "{n}"),
            ast::Expr::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            ast::Expr::LetIn {
                ident,
                value,
                child,
            } => {
                write!(f, "let {ident} := {value} in")?;
                writeln!(f)?;
                write!(f, "{}", &child)?;
                Ok(())
            }
        }
    }
}

/// Wrapper struct to print values with each line indented by the given string.
struct Indent<T> {
    val: T,
    indent: &'static str,
}

impl<T> Indent<T> {
    pub fn tab(val: T) -> Indent<T> {
        Indent { val, indent: "\t" }
    }
}

impl<T: fmt::Display> fmt::Display for Indent<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: efficient, streaming implementation of this function.
        let buf = format!("{}", self.val);
        for line in buf.lines() {
            write!(f, "{}", self.indent)?;
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}
