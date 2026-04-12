//! Recursive descent parser for flux-ese.

use crate::ast::*;
use crate::lexer::{Lexer, Token};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        t
    }

    fn eat(&mut self, expected: &Token) -> bool {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_newlines_and_comments(&mut self) {
        loop {
            match self.peek() {
                Token::Newline | Token::Comment(_) => { self.advance(); }
                _ => break,
            }
        }
    }

    pub fn parse(&mut self) -> Result<FluxProgram, String> {
        let mut setup = Vec::new();
        let mut cycles = Vec::new();

        self.skip_newlines_and_comments();

        // Parse optional setup block
        if matches!(self.peek(), Token::Setup) {
            setup = self.parse_setup()?;
        }

        self.skip_newlines_and_comments();

        // Parse on every cycle block
        if matches!(self.peek(), Token::On) {
            cycles = self.parse_cycle_block()?;
        }

        Ok(FluxProgram { setup, cycles })
    }

    fn parse_setup(&mut self) -> Result<Vec<(String, Expr)>, String> {
        self.advance(); // eat 'setup'
        self.eat(&Token::Colon);
        self.skip_newlines_and_comments();

        let mut vars = Vec::new();
        loop {
            match self.peek() {
                Token::Ident(_) => {
                    let name = match self.advance() {
                        Token::Ident(s) => s,
                        _ => unreachable!(),
                    };
                    self.eat(&Token::Assign);
                    let expr = self.parse_expr()?;
                    vars.push((name, expr));
                    self.skip_newlines_and_comments();
                }
                Token::On | Token::Eof => break,
                _ => break,
            }
        }
        Ok(vars)
    }

    fn parse_cycle_block(&mut self) -> Result<Vec<BlockItem>, String> {
        self.advance(); // eat 'on'
        self.eat(&Token::Every);
        self.eat(&Token::Cycle);
        self.eat(&Token::Colon);
        self.skip_newlines_and_comments();

        self.parse_block_items(0)
    }

    fn parse_block_items(&mut self, min_indent: usize) -> Result<Vec<BlockItem>, String> {
        let mut items = Vec::new();
        loop {
            self.skip_newlines_and_comments();
            match self.peek() {
                Token::Eof | Token::Else => break,
                Token::On | Token::Setup => break,
                _ => {
                    match self.parse_block_item(min_indent)? {
                        Some(item) => items.push(item),
                        None => break,
                    }
                }
            }
        }
        Ok(items)
    }

    fn parse_block_item(&mut self, min_indent: usize) -> Result<Option<BlockItem>, String> {
        match self.peek().clone() {
            Token::If => {
                let item = self.parse_if_stmt()?;
                Ok(Some(BlockItem::If { cond: item.cond, then: item.then, else_: item.else_ }))
            }
            Token::Read => {
                self.advance();
                let ident = match self.advance() {
                    Token::Ident(s) => s,
                    t => return Err(format!("expected identifier after 'read', got {:?}", t)),
                };
                Ok(Some(BlockItem::Read { ident }))
            }
            Token::Delegate => {
                self.advance();
                let task = match self.advance() {
                    Token::Ident(s) => s,
                    t => return Err(format!("expected task after 'delegate', got {:?}", t)),
                };
                // eat 'to'
                let _ = self.advance(); // 'to'
                let to = self.parse_expr()?;
                Ok(Some(BlockItem::Delegate { task, to }))
            }
            Token::Reply => {
                self.advance();
                let msg = match self.advance() {
                    Token::StringLit(s) => s,
                    t => return Err(format!("expected string after 'reply', got {:?}", t)),
                };
                Ok(Some(BlockItem::Reply { message: msg }))
            }
            Token::Process => {
                self.advance();
                let task = match self.advance() {
                    Token::Ident(s) => s,
                    t => return Err(format!("expected task after 'process', got {:?}", t)),
                };
                Ok(Some(BlockItem::Process { task }))
            }
            Token::TrustOf => {
                // trust_of(x) > y is handled as an expression in if conditions
                Err("trust_of must appear inside an if condition".into())
            }
            Token::Ident(ref name) if name == "instinct" => {
                // instinct.modulate(...)
                self.advance(); // instinct
                self.eat(&Token::Dot);
                let method = match self.advance() {
                    Token::Ident(s) => s,
                    t => return Err(format!("expected method name, got {:?}", t)),
                };
                self.eat(&Token::LParen);
                let name_expr = self.parse_expr()?;
                let mut params = Vec::new();
                while matches!(self.peek(), Token::Comma) {
                    self.advance();
                    if let Token::Ident(key) = self.advance() {
                        self.eat(&Token::Colon);
                        let val = self.parse_expr()?;
                        params.push((key, val));
                    }
                }
                self.eat(&Token::RParen);
                match method.as_str() {
                    "modulate" => Ok(Some(BlockItem::InstModulate { name: name_expr, params })),
                    _ => Err(format!("unknown instinct method: {}", method)),
                }
            }
            Token::Ident(_) => {
                // Assignment: ident.field = expr  or  ident = expr
                let expr = self.parse_expr()?;
                self.skip_newlines_and_comments();
                if self.eat(&Token::Assign) {
                    let value = self.parse_expr()?;
                    Ok(Some(BlockItem::Assign { target: expr, value }))
                } else {
                    Err("expected '=' in assignment".into())
                }
            }
            _ => Ok(None),
        }
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, String> {
        self.advance(); // eat 'if'
        let cond = self.parse_expr()?;
        self.eat(&Token::Colon);
        self.skip_newlines_and_comments();

        let then = self.parse_block_items(0)?;

        self.skip_newlines_and_comments();
        let else_ = if self.eat(&Token::Else) {
            self.eat(&Token::Colon);
            self.skip_newlines_and_comments();
            self.parse_block_items(0)?
        } else {
            Vec::new()
        };

        Ok(Stmt::If { cond, then, else_ })
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let left = self.parse_additive()?;

        let op = match self.peek() {
            Token::Lt => BinOp::Lt,
            Token::Gt => BinOp::Gt,
            Token::Eq => BinOp::Eq,
            Token::Ne => BinOp::Ne,
            Token::Le => BinOp::Le,
            Token::Ge => BinOp::Ge,
            _ => return Ok(left),
        };

        self.advance();
        let right = self.parse_additive()?;
        Ok(Expr::BinOp { left: Box::new(left), op, right: Box::new(right) })
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinOp { left: Box::new(left), op, right: Box::new(right) };
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;

        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_primary()?;
            left = Expr::BinOp { left: Box::new(left), op, right: Box::new(right) };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Token::Float(f) => Ok(Expr::Float(f)),
            Token::Int(i) => Ok(Expr::Int(i)),
            Token::StringLit(s) => Ok(Expr::StringLit(s)),
            Token::Ident(name) => {
                if self.peek() == &Token::Dot {
                    self.advance(); // eat dot
                    let field = match self.advance() {
                        Token::Ident(f) => f,
                        t => return Err(format!("expected field name after '.', got {:?}", t)),
                    };
                    Ok(Expr::DotAccess { obj: name, field })
                } else if self.peek() == &Token::LParen {
                    self.advance();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RParen) {
                        args.push(self.parse_expr()?);
                        while matches!(self.peek(), Token::Comma) {
                            self.advance();
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.eat(&Token::RParen);
                    Ok(Expr::Call { func: name, args })
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            Token::TrustOf => {
                self.eat(&Token::LParen);
                let arg = self.parse_expr()?;
                self.eat(&Token::RParen);
                Ok(Expr::Call { func: "trust_of".into(), args: vec![arg] })
            }
            Token::LParen => {
                let expr = self.parse_expr()?;
                self.eat(&Token::RParen);
                Ok(expr)
            }
            t => Err(format!("unexpected token in expression: {:?}", t)),
        }
    }
}
