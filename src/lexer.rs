//! Tokenizer for flux-ese.

use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Setup,
    On,
    Every,
    Cycle,
    If,
    Else,
    Delegate,
    Reply,
    Process,
    Read,
    TrustOf,
    // Literals & identifiers
    Ident(String),
    Float(f64),
    Int(i64),
    StringLit(String),
    // Operators
    Lt,
    Gt,
    Eq,
    Ne,
    Le,
    Ge,
    Star,
    Plus,
    Minus,
    Slash,
    Assign,
    Dot,
    Comma,
    Colon,
    LParen,
    RParen,
    // Special
    Comment(String),
    Newline,
    Indent(usize),
    Eof,
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { chars: input.chars().peekable(), line: 1 }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.chars.next();
        if c == Some('\n') { self.line += 1; }
        c
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => { tokens.push(Token::Eof); break; }
                Some('\n') => { self.next(); tokens.push(Token::Newline); }
                Some('#') => { tokens.push(self.read_comment()); }
                Some('"') => { tokens.push(self.read_string()); }
                Some(c) if c.is_ascii_digit() => { tokens.push(self.read_number()); }
                Some(c) if c.is_ascii_alphabetic() || c == '_' => { tokens.push(self.read_ident()); }
                Some('<') => { self.next(); if self.peek() == Some('=') { self.next(); tokens.push(Token::Le); } else { tokens.push(Token::Lt); } }
                Some('>') => { self.next(); if self.peek() == Some('=') { self.next(); tokens.push(Token::Ge); } else { tokens.push(Token::Gt); } }
                Some('=') => { self.next(); if self.peek() == Some('=') { self.next(); tokens.push(Token::Eq); } else { tokens.push(Token::Assign); } }
                Some('!') => { self.next(); self.next(); tokens.push(Token::Ne); }
                Some('*') => { self.next(); tokens.push(Token::Star); }
                Some('+') => { self.next(); tokens.push(Token::Plus); }
                Some('-') => { self.next(); tokens.push(Token::Minus); }
                Some('/') => { self.next(); tokens.push(Token::Slash); }
                Some('.') => { self.next(); tokens.push(Token::Dot); }
                Some(',') => { self.next(); tokens.push(Token::Comma); }
                Some(':') => { self.next(); tokens.push(Token::Colon); }
                Some('(') => { self.next(); tokens.push(Token::LParen); }
                Some(')') => { self.next(); tokens.push(Token::RParen); }
                Some(_) => { self.next(); } // skip unknown
            }
        }
        tokens
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == ' ' || c == '\t' || c == '\r' { self.next(); } else { break; }
        }
    }

    fn read_comment(&mut self) -> Token {
        let mut s = String::new();
        while let Some(c) = self.next() { if c == '\n' { break; } s.push(c); }
        Token::Comment(s.trim().to_string())
    }

    fn read_string(&mut self) -> Token {
        self.next(); // opening "
        let mut s = String::new();
        while let Some(c) = self.next() {
            if c == '"' { break; }
            if c == '\\' { if let Some(esc) = self.next() { s.push(esc); } } else { s.push(c); }
        }
        Token::StringLit(s)
    }

    fn read_number(&mut self) -> Token {
        let mut s = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() || c == '.' { s.push(c); self.next(); } else { break; }
        }
        if s.contains('.') { Token::Float(s.parse().unwrap()) } else { Token::Int(s.parse().unwrap()) }
    }

    fn read_ident(&mut self) -> Token {
        let mut s = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_alphanumeric() || c == '_' { s.push(c); self.next(); } else { break; }
        }
        match s.as_str() {
            "setup" => Token::Setup,
            "on" => Token::On,
            "every" => Token::Every,
            "cycle" => Token::Cycle,
            "if" => Token::If,
            "else" => Token::Else,
            "delegate" => Token::Delegate,
            "reply" => Token::Reply,
            "process" => Token::Process,
            "read" => Token::Read,
            "trust_of" => Token::TrustOf,
            _ => Token::Ident(s),
        }
    }
}
