//! flux-ese — markdown-like DSL that compiles to FLUX VM bytecodes.

pub mod ast;
pub mod compiler;
pub mod lexer;
pub mod opcodes;
pub mod parser;

use compiler::Compiler;
use parser::Parser;

/// Parse flux-ese source and compile to FLUX bytecodes.
pub fn compile(source: &str) -> Result<Vec<u8>, String> {
    let mut parser = Parser::new(source);
    let program = parser.parse()?;
    Ok(Compiler::compile(&program))
}

/// Parse flux-ese source into an AST without compiling.
pub fn parse(source: &str) -> Result<ast::FluxProgram, String> {
    let mut parser = Parser::new(source);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_simple() {
        let src = r#"
setup:
  x = 42

on every cycle:
  read energy_level
"#;
        let bc = compile(src).unwrap();
        assert!(!bc.is_empty());
        assert_eq!(bc[bc.len()-1], opcodes::Opcode::HALT.to_byte());
    }

    #[test]
    fn compile_if_else() {
        let src = r#"
on every cycle:
  if energy_level < 20:
    reply "low"
  else:
    reply "ok"
"#;
        let bc = compile(src).unwrap();
        assert!(!bc.is_empty());
    }

    #[test]
    fn compile_confidence() {
        let src = r#"
on every cycle:
  confidence.score = confidence.score * 0.95
"#;
        let bc = compile(src).unwrap();
        assert!(bc.contains(&opcodes::Opcode::CONF_MUL.to_byte()));
    }

    #[test]
    fn compile_trust() {
        let src = r#"
on every cycle:
  if trust_of(requester) > 0.7:
    reply "trusted"
"#;
        let bc = compile(src).unwrap();
        assert!(bc.contains(&opcodes::Opcode::TRUST_COMPARE.to_byte()));
    }

    #[test]
    fn compile_instinct() {
        let src = r#"
on every cycle:
  instinct.modulate("survival", urgency: 0.9)
"#;
        let bc = compile(src).unwrap();
        assert!(bc.contains(&opcodes::Opcode::INST_MODULATE.to_byte()));
    }

    #[test]
    fn compile_delegate() {
        let src = r#"
on every cycle:
  delegate task to requester
"#;
        let bc = compile(src).unwrap();
        assert!(bc.contains(&opcodes::Opcode::DELEGATE.to_byte()));
    }

    #[test]
    fn compile_setup() {
        let src = r#"
setup:
  trust_threshold = 0.7
  energy_warning = 20
"#;
        let bc = compile(src).unwrap();
        assert!(bc.contains(&opcodes::Opcode::SETUP_CONST.to_byte()));
    }

    #[test]
    fn compile_nested_if() {
        let src = r#"
on every cycle:
  if energy_level < 20:
    if trust_of(requester) > 0.7:
      delegate task to requester
    else:
      reply "no trust"
  else:
    reply "ok"
"#;
        let bc = compile(src).unwrap();
        assert!(bc.contains(&opcodes::Opcode::DELEGATE.to_byte()));
        assert!(bc.contains(&opcodes::Opcode::REPLY.to_byte()));
    }

    #[test]
    fn full_program() {
        let src = r#"
setup:
  trust_threshold = 0.7
  energy_warning = 20

on every cycle:
  read energy_level
  if energy_level < energy_warning:
    confidence.score = confidence.score * 0.95
    instinct.modulate("survival", urgency: 0.9)
    if trust_of(requester) > trust_threshold:
      delegate task to requester
    else:
      reply "insufficient trust"
  else:
    process task
    confidence.score = confidence.score * 0.99
"#;
        let bc = compile(src).unwrap();
        assert_eq!(bc[bc.len()-1], opcodes::Opcode::HALT.to_byte());
        // Verify key opcodes present
        assert!(bc.contains(&opcodes::Opcode::ENERGY_REPORT.to_byte()));
        assert!(bc.contains(&opcodes::Opcode::CONF_MUL.to_byte()));
        assert!(bc.contains(&opcodes::Opcode::INST_MODULATE.to_byte()));
        assert!(bc.contains(&opcodes::Opcode::DELEGATE.to_byte()));
        assert!(bc.contains(&opcodes::Opcode::REPLY.to_byte()));
        assert!(bc.contains(&opcodes::Opcode::PROCESS_TASK.to_byte()));
    }

    #[test]
    fn parse_ast() {
        let src = r#"setup:
  x = 10
on every cycle:
  reply "hello""#;
        let prog = parse(src).unwrap();
        assert_eq!(prog.setup.len(), 1);
        assert_eq!(prog.cycles.len(), 1);
    }
}
