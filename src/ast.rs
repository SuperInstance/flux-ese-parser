//! AST types for flux-ese.

#[derive(Debug, Clone)]
pub enum Expr {
    Float(f64),
    Int(i64),
    Ident(String),
    DotAccess { obj: String, field: String },
    BinOp { left: Box<Expr>, op: BinOp, right: Box<Expr> },
    Call { func: String, args: Vec<Expr> },
    StringLit(String),
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Lt,
    Gt,
    Eq,
    Ne,
    Le,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Comment(String),
    Setup(Vec<(String, Expr)>),
    OnEveryCycle(Vec<BlockItem>),
    If { cond: Expr, then: Vec<BlockItem>, else_: Vec<BlockItem> },
    Assign { target: Expr, value: Expr },
    Read { ident: String },
    Delegate { task: String, to: Expr },
    Reply { message: String },
    Process { task: String },
    InstModulate { name: Expr, params: Vec<(String, Expr)> },
}

#[derive(Debug, Clone)]
pub enum BlockItem {
    If { cond: Expr, then: Vec<BlockItem>, else_: Vec<BlockItem> },
    Assign { target: Expr, value: Expr },
    Read { ident: String },
    Delegate { task: String, to: Expr },
    Reply { message: String },
    Process { task: String },
    InstModulate { name: Expr, params: Vec<(String, Expr)> },
}

impl From<Stmt> for BlockItem {
    fn from(s: Stmt) -> Self {
        match s {
            Stmt::If { cond, then, else_ } => BlockItem::If { cond, then, else_ },
            Stmt::Assign { target, value } => BlockItem::Assign { target, value },
            Stmt::Read { ident } => BlockItem::Read { ident },
            Stmt::Delegate { task, to } => BlockItem::Delegate { task, to },
            Stmt::Reply { message } => BlockItem::Reply { message },
            Stmt::Process { task } => BlockItem::Process { task },
            Stmt::InstModulate { name, params } => BlockItem::InstModulate { name, params },
            Stmt::Comment(_) | Stmt::Setup(_) | Stmt::OnEveryCycle(_) => panic!("invalid block item"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FluxProgram {
    pub setup: Vec<(String, Expr)>,
    pub cycles: Vec<BlockItem>,
}
