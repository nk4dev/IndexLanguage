//! Recursive-descent parser for IndexLanguage.

use crate::ast::*;
use crate::lexer::{Tok, Token};

pub fn parse(tokens: Vec<Token>) -> Result<Program, String> {
    let mut p = Parser { tokens, pos: 0 };
    let mut items = Vec::new();
    while !p.check(&Tok::Eof) {
        items.push(p.statement()?);
    }
    Ok(Program { items })
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn line(&self) -> usize {
        self.peek().line
    }

    fn check(&self, t: &Tok) -> bool {
        &self.peek().tok == t
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn eat(&mut self, t: &Tok, what: &str) -> Result<Token, String> {
        if self.check(t) {
            Ok(self.advance())
        } else {
            Err(format!(
                "line {}: expected {}, found {:?}",
                self.line(),
                what,
                self.peek().tok
            ))
        }
    }

    fn match_tok(&mut self, t: &Tok) -> bool {
        if self.check(t) {
            self.advance();
            true
        } else {
            false
        }
    }

    // ---- statements ----

    fn statement(&mut self) -> Result<Stmt, String> {
        match &self.peek().tok {
            Tok::Fn => self.fn_decl(),
            Tok::Let | Tok::Const => self.let_stmt(),
            Tok::Return => self.return_stmt(),
            Tok::If => self.if_stmt(),
            Tok::While => self.while_stmt(),
            Tok::LBrace => Ok(Stmt::Block(self.block()?)),
            _ => {
                let expr = self.expression()?;
                self.eat(&Tok::Semi, "';'")?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        self.eat(&Tok::LBrace, "'{'")?;
        let mut stmts = Vec::new();
        while !self.check(&Tok::RBrace) && !self.check(&Tok::Eof) {
            stmts.push(self.statement()?);
        }
        self.eat(&Tok::RBrace, "'}'")?;
        Ok(stmts)
    }

    fn fn_decl(&mut self) -> Result<Stmt, String> {
        let line = self.line();
        self.eat(&Tok::Fn, "'fn'")?;
        let name = self.ident()?;
        self.eat(&Tok::LParen, "'('")?;
        let mut params = Vec::new();
        if !self.check(&Tok::RParen) {
            loop {
                let pname = self.ident()?;
                self.eat(&Tok::Colon, "':'")?;
                let ty = self.type_ref()?;
                params.push(Param { name: pname, ty });
                if !self.match_tok(&Tok::Comma) {
                    break;
                }
            }
        }
        self.eat(&Tok::RParen, "')'")?;
        let ret = if self.match_tok(&Tok::Colon) {
            self.type_ref()?
        } else {
            Type::Void
        };
        let body = self.block()?;
        Ok(Stmt::Fn(FnDecl {
            name,
            params,
            ret,
            body,
            line,
        }))
    }

    fn let_stmt(&mut self) -> Result<Stmt, String> {
        let line = self.line();
        let is_const = matches!(self.peek().tok, Tok::Const);
        self.advance();
        let name = self.ident()?;
        let ty = if self.match_tok(&Tok::Colon) {
            Some(self.type_ref()?)
        } else {
            None
        };
        self.eat(&Tok::Assign, "'='")?;
        let init = self.expression()?;
        self.eat(&Tok::Semi, "';'")?;
        Ok(Stmt::Let {
            is_const,
            name,
            ty,
            init,
            line,
        })
    }

    fn return_stmt(&mut self) -> Result<Stmt, String> {
        let line = self.line();
        self.eat(&Tok::Return, "'return'")?;
        let value = if self.check(&Tok::Semi) {
            None
        } else {
            Some(self.expression()?)
        };
        self.eat(&Tok::Semi, "';'")?;
        Ok(Stmt::Return { value, line })
    }

    fn if_stmt(&mut self) -> Result<Stmt, String> {
        self.eat(&Tok::If, "'if'")?;
        let cond = self.expression()?;
        let then_branch = self.block()?;
        let else_branch = if self.match_tok(&Tok::Else) {
            if self.check(&Tok::If) {
                Some(vec![self.if_stmt()?])
            } else {
                Some(self.block()?)
            }
        } else {
            None
        };
        Ok(Stmt::If {
            cond,
            then_branch,
            else_branch,
        })
    }

    fn while_stmt(&mut self) -> Result<Stmt, String> {
        self.eat(&Tok::While, "'while'")?;
        let cond = self.expression()?;
        let body = self.block()?;
        Ok(Stmt::While { cond, body })
    }

    // ---- types ----

    fn type_ref(&mut self) -> Result<Type, String> {
        let mut base = match &self.peek().tok {
            Tok::KwNumber => {
                self.advance();
                Type::Number
            }
            Tok::KwString => {
                self.advance();
                Type::String
            }
            Tok::KwBool => {
                self.advance();
                Type::Bool
            }
            Tok::KwVoid => {
                self.advance();
                Type::Void
            }
            Tok::KwAny => {
                self.advance();
                Type::Any
            }
            Tok::Ident(name) => {
                let n = name.clone();
                self.advance();
                Type::Named(n)
            }
            other => {
                return Err(format!("line {}: expected a type, found {:?}", self.line(), other))
            }
        };
        while self.check(&Tok::LBracket) {
            self.advance();
            self.eat(&Tok::RBracket, "']'")?;
            base = Type::Array(Box::new(base));
        }
        Ok(base)
    }

    fn ident(&mut self) -> Result<String, String> {
        match &self.peek().tok {
            Tok::Ident(name) => {
                let n = name.clone();
                self.advance();
                Ok(n)
            }
            other => Err(format!(
                "line {}: expected an identifier, found {:?}",
                self.line(),
                other
            )),
        }
    }

    // ---- expressions (precedence climbing) ----

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.logic_or()?;
        if self.check(&Tok::Assign) {
            let line = self.line();
            self.advance();
            let value = self.assignment()?;
            if let Expr::Ident(name, _) = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                    line,
                });
            }
            return Err(format!("line {}: invalid assignment target", line));
        }
        Ok(expr)
    }

    fn binary_level(
        &mut self,
        ops: &[(Tok, &str)],
        next: fn(&mut Self) -> Result<Expr, String>,
    ) -> Result<Expr, String> {
        let mut left = next(self)?;
        loop {
            let mut matched = None;
            for (tok, name) in ops {
                if self.check(tok) {
                    matched = Some(name.to_string());
                    break;
                }
            }
            match matched {
                Some(op) => {
                    let line = self.line();
                    self.advance();
                    let right = next(self)?;
                    left = Expr::Binary {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                        line,
                    };
                }
                None => break,
            }
        }
        Ok(left)
    }

    fn logic_or(&mut self) -> Result<Expr, String> {
        self.binary_level(&[(Tok::PipePipe, "||")], Self::logic_and)
    }

    fn logic_and(&mut self) -> Result<Expr, String> {
        self.binary_level(&[(Tok::AmpAmp, "&&")], Self::equality)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        self.binary_level(&[(Tok::EqEq, "=="), (Tok::NotEq, "!=")], Self::comparison)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        self.binary_level(
            &[
                (Tok::Lt, "<"),
                (Tok::Gt, ">"),
                (Tok::Le, "<="),
                (Tok::Ge, ">="),
            ],
            Self::term,
        )
    }

    fn term(&mut self) -> Result<Expr, String> {
        self.binary_level(&[(Tok::Plus, "+"), (Tok::Minus, "-")], Self::factor)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        self.binary_level(
            &[(Tok::Star, "*"), (Tok::Slash, "/"), (Tok::Percent, "%")],
            Self::unary,
        )
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.check(&Tok::Bang) || self.check(&Tok::Minus) {
            let line = self.line();
            let op = if self.check(&Tok::Bang) { "!" } else { "-" }.to_string();
            self.advance();
            let expr = self.unary()?;
            return Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
                line,
            });
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;
        loop {
            let line = self.line();
            if self.match_tok(&Tok::LParen) {
                let mut args = Vec::new();
                if !self.check(&Tok::RParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_tok(&Tok::Comma) {
                            break;
                        }
                    }
                }
                self.eat(&Tok::RParen, "')'")?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                    line,
                };
            } else if self.match_tok(&Tok::LBracket) {
                let index = self.expression()?;
                self.eat(&Tok::RBracket, "']'")?;
                expr = Expr::Index {
                    target: Box::new(expr),
                    index: Box::new(index),
                    line,
                };
            } else if self.match_tok(&Tok::Dot) {
                let name = self.ident()?;
                expr = Expr::Member {
                    target: Box::new(expr),
                    name,
                    line,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let line = self.line();
        match self.peek().tok.clone() {
            Tok::Number(n) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            Tok::Str(s) => {
                self.advance();
                Ok(Expr::Str(s))
            }
            Tok::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            Tok::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            Tok::Ident(name) => {
                self.advance();
                Ok(Expr::Ident(name, line))
            }
            Tok::LParen => {
                self.advance();
                let expr = self.expression()?;
                self.eat(&Tok::RParen, "')'")?;
                Ok(expr)
            }
            Tok::LBracket => {
                self.advance();
                let mut elems = Vec::new();
                if !self.check(&Tok::RBracket) {
                    loop {
                        elems.push(self.expression()?);
                        if !self.match_tok(&Tok::Comma) {
                            break;
                        }
                    }
                }
                self.eat(&Tok::RBracket, "']'")?;
                Ok(Expr::Array(elems, line))
            }
            other => Err(format!(
                "line {}: unexpected token {:?} in expression",
                line, other
            )),
        }
    }
}
