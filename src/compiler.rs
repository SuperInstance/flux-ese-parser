//! AST → FLUX bytecode compiler.

use crate::ast::*;
use crate::opcodes::Opcode;

#[derive(Debug)]
pub struct Compiler {
    bytecodes: Vec<u8>,
    strings: Vec<(usize, String)>, // (offset, string)
}

impl Compiler {
    pub fn new() -> Self {
        Compiler { bytecodes: Vec::new(), strings: Vec::new() }
    }

    pub fn compile(program: &FluxProgram) -> Vec<u8> {
        let mut c = Compiler::new();
        // Compile setup block
        c.emit_setup(&program.setup);
        // Mark cycle start
        c.emit(Opcode::NOP); // cycle entry marker
        // Compile cycle body
        c.compile_block(&program.cycles);
        c.emit(Opcode::HALT);
        c.bytecodes
    }

    fn emit(&mut self, op: Opcode) { self.bytecodes.push(op.to_byte()); }
    fn emit_u8(&mut self, v: u8) { self.bytecodes.push(v); }
    fn emit_u16(&mut self, v: u16) { self.bytecodes.extend_from_slice(&v.to_le_bytes()); }
    fn emit_f64(&mut self, v: f64) { self.bytecodes.extend_from_slice(&v.to_le_bytes()); }

    fn current_offset(&self) -> usize { self.bytecodes.len() }

    fn patch_u16(&mut self, offset: usize, v: u16) {
        self.bytecodes[offset..offset+2].copy_from_slice(&v.to_le_bytes());
    }

    fn store_string(&mut self, s: &str) -> u16 {
        let offset = self.bytecodes.len() as u16;
        self.strings.push((offset as usize, s.to_string()));
        self.emit(Opcode::STORE_STRING);
        let len = s.len() as u16;
        self.emit_u16(len);
        for b in s.as_bytes() { self.emit_u8(*b); }
        offset
    }

    fn resolve_ident(&self, name: &str) -> u8 {
        match name {
            "energy_level" => 0,
            "trust_threshold" => 1,
            "energy_warning" => 2,
            "confidence_score" => 3,
            _ => (name.len() % 16) as u8,
        }
    }

    fn emit_setup(&mut self, setup: &[(String, Expr)]) {
        for (name, expr) in setup {
            let reg = self.resolve_ident(name);
            self.emit(Opcode::SETUP_CONST);
            self.emit_u8(reg);
            match expr {
                Expr::Float(f) => self.emit_f64(*f),
                Expr::Int(i) => self.emit_f64(*i as f64),
                _ => self.emit_f64(0.0),
            }
        }
    }

    fn compile_block(&mut self, items: &[BlockItem]) {
        for item in items {
            self.compile_block_item(item);
        }
    }

    fn compile_block_item(&mut self, item: &BlockItem) {
        match item {
            BlockItem::If { cond, then, else_ } => self.compile_if(cond, then, else_),
            BlockItem::Assign { target, value } => self.compile_assign(target, value),
            BlockItem::Read { ident } => self.compile_read(ident),
            BlockItem::Delegate { task, to } => self.compile_delegate(task, to),
            BlockItem::Reply { message } => self.compile_reply(message),
            BlockItem::Process { task } => self.compile_process(task),
            BlockItem::InstModulate { name, params } => self.compile_inst_modulate(name, params),
        }
    }

    fn compile_if(&mut self, cond: &Expr, then: &[BlockItem], else_: &[BlockItem]) {
        // Compile condition → set flags
        self.compile_expr_for_cmp(cond);

        let else_jmp_offset = self.current_offset();
        // Placeholder: JLT/JGT/JEQ depending on condition
        let (op, swapped) = match cond {
            Expr::BinOp { op: BinOp::Lt, .. } => (Opcode::JLT, false),
            Expr::BinOp { op: BinOp::Gt, .. } => (Opcode::JGT, false),
            Expr::BinOp { op: BinOp::Eq, .. } => (Opcode::JEQ, false),
            Expr::BinOp { op: BinOp::Ne, .. } => (Opcode::JNE, false),
            Expr::BinOp { op: BinOp::Le, .. } => (Opcode::JLE, false),
            Expr::BinOp { op: BinOp::Ge, .. } => (Opcode::JGE, false),
            _ => (Opcode::JLT, false),
        };

        self.emit(Opcode::CMP);
        self.emit_u8(0); // reg a
        self.emit_u8(1); // reg b

        // Jump past then-block to else (or past both)
        self.emit(op);
        let jmp_patch = self.current_offset();
        self.emit_u16(0); // placeholder

        // Compile then-block
        self.compile_block(then);

        if !else_.is_empty() {
            // Jump past else-block
            self.emit(Opcode::JMP);
            let end_patch = self.current_offset();
            self.emit_u16(0);
            // Patch the condition jump to here (start of else)
            self.patch_u16(jmp_patch, self.current_offset() as u16);
            self.compile_block(else_);
            // Patch end jump
            self.patch_u16(end_patch, self.current_offset() as u16);
        } else {
            self.patch_u16(jmp_patch, self.current_offset() as u16);
        }
    }

    fn compile_expr_for_cmp(&mut self, expr: &Expr) {
        match expr {
            Expr::BinOp { left, op: _, right } => {
                // Load left into R0, right into R1
                self.load_expr(left, 0);
                self.load_expr(right, 1);
            }
            _ => {
                self.load_expr(expr, 0);
                self.emit(Opcode::LOAD_CONST);
                self.emit_u8(1);
                self.emit_f64(0.0);
            }
        }
    }

    fn load_expr(&mut self, expr: &Expr, reg: u8) {
        match expr {
            Expr::Call { func, args } if func == "trust_of" => {
                self.emit(Opcode::TRUST_COMPARE);
                self.emit_u8(reg);
                if let Some(arg) = args.first() {
                    self.load_expr(arg, reg);
                }
            }
            Expr::Float(f) => {
                self.emit(Opcode::LOAD_CONST);
                self.emit_u8(reg);
                self.emit_f64(*f);
            }
            Expr::Int(i) => {
                self.emit(Opcode::LOAD_CONST);
                self.emit_u8(reg);
                self.emit_f64(*i as f64);
            }
            Expr::Ident(name) => {
                self.emit(Opcode::LOAD_REG);
                self.emit_u8(reg);
                self.emit_u8(self.resolve_ident(name));
            }
            Expr::BinOp { left, op, right } => {
                self.load_expr(left, reg);
                self.load_expr(right, reg.wrapping_add(1));
                let op_byte = match op {
                    BinOp::Add => Opcode::ADD,
                    BinOp::Sub => Opcode::SUB,
                    BinOp::Mul => Opcode::MUL,
                    BinOp::Div => Opcode::DIV,
                    _ => Opcode::ADD,
                };
                self.emit(op_byte);
                self.emit_u8(reg);
            }
            Expr::DotAccess { obj, field } if obj == "confidence" && field == "score" => {
                self.emit(Opcode::CONF_GET);
                self.emit_u8(reg);
            }
            _ => {
                self.emit(Opcode::LOAD_CONST);
                self.emit_u8(reg);
                self.emit_f64(0.0);
            }
        }
    }

    fn compile_assign(&mut self, target: &Expr, value: &Expr) {
        // Special case: confidence.score = expr
        if let Expr::DotAccess { obj, field } = target {
            if obj == "confidence" && field == "score" {
                // Load current confidence
                self.emit(Opcode::CONF_GET);
                self.emit_u8(0);
                // If value involves multiplication (decay), load the factor
                if let Expr::BinOp { left: _, op: BinOp::Mul, right } = value {
                    if let Expr::DotAccess { obj, field } = left {
                        if obj == "confidence" && field == "score" {
                            // confidence.score * factor
                            self.load_expr(right, 1);
                            self.emit(Opcode::CONF_MUL);
                            self.emit_u8(0);
                            return;
                        }
                    }
                }
                // Generic: load value, multiply with confidence, set
                self.load_expr(value, 1);
                self.emit(Opcode::CONF_MUL);
                self.emit_u8(0);
                return;
            }
        }
        // Generic register assign
        self.load_expr(value, 0);
        self.emit(Opcode::STORE_REG);
        self.emit_u8(0);
        if let Expr::Ident(name) = target {
            self.emit_u8(self.resolve_ident(name));
        } else {
            self.emit_u8(0);
        }
    }

    fn compile_read(&mut self, ident: &str) {
        if ident == "energy_level" {
            self.emit(Opcode::ENERGY_REPORT);
            self.emit_u8(0); // store in R0
        } else {
            self.emit(Opcode::READ_SENSOR);
            self.emit_u8(self.resolve_ident(ident));
        }
    }

    fn compile_delegate(&mut self, task: &str, to: &Expr) {
        self.store_string(task);
        if let Expr::Ident(name) = to {
            self.emit(Opcode::DELEGATE);
            self.emit_u8(self.resolve_ident(name));
        } else {
            self.emit(Opcode::DELEGATE);
            self.emit_u8(0);
        }
    }

    fn compile_reply(&mut self, message: &str) {
        self.store_string(message);
        self.emit(Opcode::REPLY);
        self.emit_u8(0);
    }

    fn compile_process(&mut self, task: &str) {
        self.store_string(task);
        self.emit(Opcode::PROCESS_TASK);
        self.emit_u8(0);
    }

    fn compile_inst_modulate(&mut self, name: &Expr, params: &[(String, Expr)]) {
        self.store_string("instinct");
        // Store the name string
        if let Expr::StringLit(s) = name {
            self.store_string(s);
        }
        self.emit(Opcode::INST_MODULATE);
        self.emit_u8(0); // mode register
        // Encode urgency if present
        for (key, val) in params {
            if key == "urgency" {
                self.load_expr(val, 0);
            }
        }
    }
}
