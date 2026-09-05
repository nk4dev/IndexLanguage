//! Lightweight type checker for IndexLanguage.
//!
//! This is deliberately permissive (gradual typing): `any` silences checks, and
//! unknown named types are treated as `any`. It catches the mistakes that matter
//! in practice: unknown names, arity errors, `const` reassignment, bad operand
//! types, and return/annotation mismatches.

use crate::ast::*;
use std::collections::HashMap;

struct Checker {
    scopes: Vec<HashMap<String, Binding>>,
    fns: HashMap<String, FnSig>,
    current_ret: Vec<Type>,
    errors: Vec<String>,
}

#[derive(Clone)]
struct Binding {
    ty: Type,
    is_const: bool,
}

#[derive(Clone)]
struct FnSig {
    params: Vec<Type>,
    ret: Type,
}

pub fn check(program: &Program) -> Result<(), String> {
    let mut c = Checker {
        scopes: vec![HashMap::new()],
        fns: HashMap::new(),
        current_ret: Vec::new(),
        errors: Vec::new(),
    };

    // hoist function signatures
    for item in &program.items {
        if let Stmt::Fn(f) = item {
            if c.fns.contains_key(&f.name) {
                c.errors
                    .push(format!("line {}: function '{}' is already defined", f.line, f.name));
            }
            c.fns.insert(
                f.name.clone(),
                FnSig {
                    params: f.params.iter().map(|p| p.ty.clone()).collect(),
                    ret: f.ret.clone(),
                },
            );
        }
    }

    for item in &program.items {
        c.stmt(item);
    }

    if c.errors.is_empty() {
        Ok(())
    } else {
        Err(c.errors.join("\n"))
    }
}

fn normalize(t: &Type) -> Type {
    match t {
        Type::Named(_) => Type::Any,
        Type::Array(inner) => Type::Array(Box::new(normalize(inner))),
        other => other.clone(),
    }
}

/// Is a value of type `from` acceptable where `to` is expected?
fn assignable(from: &Type, to: &Type) -> bool {
    let from = normalize(from);
    let to = normalize(to);
    if from == Type::Any || to == Type::Any {
        return true;
    }
    match (&from, &to) {
        (Type::Array(a), Type::Array(b)) => assignable(a, b),
        _ => from == to,
    }
}

impl Checker {
    fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn pop(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: &str, ty: Type, is_const: bool) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), Binding { ty, is_const });
    }

    fn lookup(&self, name: &str) -> Option<Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope.get(name) {
                return Some(b.clone());
            }
        }
        None
    }

    fn err(&mut self, line: usize, msg: impl Into<String>) {
        self.errors.push(format!("line {}: {}", line, msg.into()));
    }

    fn block(&mut self, stmts: &[Stmt]) {
        self.push();
        for s in stmts {
            self.stmt(s);
        }
        self.pop();
    }

    fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Fn(f) => {
                self.push();
                for p in &f.params {
                    self.define(&p.name, p.ty.clone(), false);
                }
                self.current_ret.push(f.ret.clone());
                for s in &f.body {
                    self.stmt(s);
                }
                self.current_ret.pop();
                self.pop();
            }
            Stmt::Let {
                is_const,
                name,
                ty,
                init,
                line,
            } => {
                let init_ty = self.expr(init);
                let final_ty = match ty {
                    Some(annotated) => {
                        if !assignable(&init_ty, annotated) {
                            self.err(
                                *line,
                                format!(
                                    "cannot assign {} to '{}: {}'",
                                    init_ty.render(),
                                    name,
                                    annotated.render()
                                ),
                            );
                        }
                        annotated.clone()
                    }
                    None => init_ty,
                };
                self.define(name, final_ty, *is_const);
            }
            Stmt::Return { value, line } => {
                let expected = self.current_ret.last().cloned();
                match expected {
                    None => self.err(*line, "'return' outside of a function"),
                    Some(ret_ty) => {
                        let got = match value {
                            Some(e) => self.expr(e),
                            None => Type::Void,
                        };
                        if !assignable(&got, &ret_ty) {
                            self.err(
                                *line,
                                format!(
                                    "returning {} from a function declared {}",
                                    got.render(),
                                    ret_ty.render()
                                ),
                            );
                        }
                    }
                }
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.expect_bool(cond);
                self.block(then_branch);
                if let Some(else_b) = else_branch {
                    self.block(else_b);
                }
            }
            Stmt::While { cond, body } => {
                self.expect_bool(cond);
                self.block(body);
            }
            Stmt::Expr(e) => {
                self.expr(e);
            }
            Stmt::Block(stmts) => self.block(stmts),
        }
    }

    fn expect_bool(&mut self, cond: &Expr) {
        let t = self.expr(cond);
        let n = normalize(&t);
        if n != Type::Bool && n != Type::Any {
            let line = expr_line(cond);
            self.err(line, format!("condition must be bool, found {}", t.render()));
        }
    }

    fn expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Number(_) => Type::Number,
            Expr::Str(_) => Type::String,
            Expr::Bool(_) => Type::Bool,
            Expr::Ident(name, line) => match self.lookup(name) {
                Some(b) => b.ty,
                None => {
                    if self.fns.contains_key(name) {
                        Type::Any
                    } else {
                        self.err(*line, format!("unknown name '{}'", name));
                        Type::Any
                    }
                }
            },
            Expr::Array(elems, _) => {
                let mut elem_ty = Type::Any;
                for (idx, e) in elems.iter().enumerate() {
                    let t = self.expr(e);
                    if idx == 0 {
                        elem_ty = t;
                    } else if !assignable(&t, &elem_ty) && !assignable(&elem_ty, &t) {
                        elem_ty = Type::Any;
                    }
                }
                Type::Array(Box::new(elem_ty))
            }
            Expr::Unary { op, expr, line } => {
                let t = self.expr(expr);
                let n = normalize(&t);
                match op.as_str() {
                    "!" => {
                        if n != Type::Bool && n != Type::Any {
                            self.err(*line, format!("'!' expects bool, found {}", t.render()));
                        }
                        Type::Bool
                    }
                    "-" => {
                        if n != Type::Number && n != Type::Any {
                            self.err(*line, format!("unary '-' expects number, found {}", t.render()));
                        }
                        Type::Number
                    }
                    _ => Type::Any,
                }
            }
            Expr::Binary {
                op,
                left,
                right,
                line,
            } => {
                let l = normalize(&self.expr(left));
                let r = normalize(&self.expr(right));
                match op.as_str() {
                    "+" => {
                        if l == Type::Any || r == Type::Any {
                            Type::Any
                        } else if l == Type::Number && r == Type::Number {
                            Type::Number
                        } else if (l == Type::String || l == Type::Number)
                            && (r == Type::String || r == Type::Number)
                        {
                            // string + number coerces to string, like TypeScript.
                            Type::String
                        } else {
                            self.err(
                                *line,
                                format!("cannot apply '+' to {} and {}", l.render(), r.render()),
                            );
                            Type::Any
                        }
                    }
                    "-" | "*" | "/" | "%" => {
                        if l != Type::Number && l != Type::Any {
                            self.err(*line, format!("'{}' expects number, found {}", op, l.render()));
                        }
                        if r != Type::Number && r != Type::Any {
                            self.err(*line, format!("'{}' expects number, found {}", op, r.render()));
                        }
                        Type::Number
                    }
                    "==" | "!=" => Type::Bool,
                    "<" | ">" | "<=" | ">=" => {
                        if l != Type::Number && l != Type::Any {
                            self.err(*line, format!("'{}' expects number, found {}", op, l.render()));
                        }
                        if r != Type::Number && r != Type::Any {
                            self.err(*line, format!("'{}' expects number, found {}", op, r.render()));
                        }
                        Type::Bool
                    }
                    "&&" | "||" => {
                        if l != Type::Bool && l != Type::Any {
                            self.err(*line, format!("'{}' expects bool, found {}", op, l.render()));
                        }
                        if r != Type::Bool && r != Type::Any {
                            self.err(*line, format!("'{}' expects bool, found {}", op, r.render()));
                        }
                        Type::Bool
                    }
                    _ => Type::Any,
                }
            }
            Expr::Assign { name, value, line } => {
                let vt = self.expr(value);
                match self.lookup(name) {
                    None => {
                        self.err(*line, format!("unknown name '{}'", name));
                    }
                    Some(b) => {
                        if b.is_const {
                            self.err(*line, format!("cannot reassign const '{}'", name));
                        }
                        if !assignable(&vt, &b.ty) {
                            self.err(
                                *line,
                                format!(
                                    "cannot assign {} to '{}: {}'",
                                    vt.render(),
                                    name,
                                    b.ty.render()
                                ),
                            );
                        }
                    }
                }
                vt
            }
            Expr::Call { callee, args, line } => {
                let arg_types: Vec<Type> = args.iter().map(|a| self.expr(a)).collect();
                if let Expr::Ident(name, _) = callee.as_ref() {
                    if name == "print" {
                        return Type::Void;
                    }
                    if let Some(sig) = self.fns.get(name).cloned() {
                        if arg_types.len() != sig.params.len() {
                            self.err(
                                *line,
                                format!(
                                    "'{}' expects {} argument(s), got {}",
                                    name,
                                    sig.params.len(),
                                    arg_types.len()
                                ),
                            );
                        } else {
                            for (i, (got, want)) in
                                arg_types.iter().zip(sig.params.iter()).enumerate()
                            {
                                if !assignable(got, want) {
                                    self.err(
                                        *line,
                                        format!(
                                            "argument {} of '{}': expected {}, got {}",
                                            i + 1,
                                            name,
                                            want.render(),
                                            got.render()
                                        ),
                                    );
                                }
                            }
                        }
                        return sig.ret.clone();
                    }
                    if self.lookup(name).is_none() {
                        self.err(*line, format!("unknown function '{}'", name));
                    }
                    return Type::Any;
                }
                self.expr(callee);
                Type::Any
            }
            Expr::Index { target, index, line } => {
                let t = self.expr(target);
                let it = normalize(&self.expr(index));
                if it != Type::Number && it != Type::Any {
                    self.err(*line, format!("index must be number, found {}", it.render()));
                }
                match normalize(&t) {
                    Type::Array(inner) => *inner,
                    Type::String => Type::String,
                    Type::Any => Type::Any,
                    other => {
                        self.err(*line, format!("cannot index {}", other.render()));
                        Type::Any
                    }
                }
            }
            Expr::Member { target, name, line } => {
                let t = self.expr(target);
                match (normalize(&t), name.as_str()) {
                    (Type::Array(_), "length") | (Type::String, "length") => Type::Number,
                    (Type::Any, _) => Type::Any,
                    (other, _) => {
                        self.err(
                            *line,
                            format!("type {} has no member '{}'", other.render(), name),
                        );
                        Type::Any
                    }
                }
            }
        }
    }
}

fn expr_line(e: &Expr) -> usize {
    match e {
        Expr::Ident(_, l)
        | Expr::Array(_, l)
        | Expr::Unary { line: l, .. }
        | Expr::Binary { line: l, .. }
        | Expr::Assign { line: l, .. }
        | Expr::Call { line: l, .. }
        | Expr::Index { line: l, .. }
        | Expr::Member { line: l, .. } => *l,
        _ => 0,
    }
}
