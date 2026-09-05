//! Abstract syntax tree for IndexLanguage (il).

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Number,
    String,
    Bool,
    Void,
    Any,
    Array(Box<Type>),
    /// A named type we don't resolve yet (treated like `Any` by the checker).
    Named(String),
}

impl Type {
    pub fn render(&self) -> String {
        match self {
            Type::Number => "number".into(),
            Type::String => "string".into(),
            Type::Bool => "bool".into(),
            Type::Void => "void".into(),
            Type::Any => "any".into(),
            Type::Array(inner) => format!("{}[]", inner.render()),
            Type::Named(n) => n.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Type,
    pub body: Vec<Stmt>,
    pub line: usize,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Fn(FnDecl),
    Let {
        is_const: bool,
        name: String,
        ty: Option<Type>,
        init: Expr,
        line: usize,
    },
    Return {
        value: Option<Expr>,
        line: usize,
    },
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Expr(Expr),
    Block(Vec<Stmt>),
}

#[derive(Clone, Debug)]
pub enum Expr {
    Number(f64),
    Str(String),
    Bool(bool),
    Ident(String, usize),
    Array(Vec<Expr>, usize),
    Unary {
        op: String,
        expr: Box<Expr>,
        line: usize,
    },
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
        line: usize,
    },
    Assign {
        name: String,
        value: Box<Expr>,
        line: usize,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        line: usize,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        line: usize,
    },
    Member {
        target: Box<Expr>,
        name: String,
        line: usize,
    },
}

pub struct Program {
    pub items: Vec<Stmt>,
}
