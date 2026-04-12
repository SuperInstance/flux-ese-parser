# flux-ese

> Markdown thoughts frozen as bytecodes.

flux-ese is a markdown-like DSL that compiles to FLUX VM bytecodes. The key insight: agents think in high-level behaviors, not individual opcodes.

## Quick Start

```rust
use flux_ese::compile;

let source = r#"
setup:
  energy_warning = 20

on every cycle:
  read energy_level
  if energy_level < energy_warning:
    reply "low energy"
"#;

let bytecodes = compile(source).unwrap();
// bytecodes: Vec<u8> ready for FLUX VM execution
```

## Syntax

### Setup Block

```fluxese
setup:
  trust_threshold = 0.7
  energy_warning = 20
```

Compiles to `SETUP_CONST` opcodes — initial values loaded at program start.

### Cycle Block

```fluxese
on every cycle:
  read energy_level
```

The main loop body. `read energy_level` compiles to `ENERGY_REPORT` into register R0.

### Conditionals

```fluxese
if energy_level < energy_warning:
  reply "low"
else:
  reply "ok"
```

Compiles to `CMP`, `JLT` (jump to else), with `JMP` past else at end of then-block.

### Confidence Operations

```fluxese
confidence.score = confidence.score * 0.95
```

Compiles to `CONF_GET`, `LOAD_CONST 0.95`, `CONF_MUL` — the confidence decay primitive.

### Trust Operations

```fluxese
if trust_of(requester) > trust_threshold:
  delegate task to requester
```

`trust_of(x)` compiles to `TRUST_COMPARE`. The `>` triggers `CMP` + `JGT`.

### Instinct Modulation

```fluxese
instinct.modulate("survival", urgency: 0.9)
```

Compiles to `INST_MODULATE` with urgency value loaded into register.

### Actions

```fluxese
delegate task to requester    # DELEGATE opcode
reply "message"               # REPLY + stored string
process task                  # PROCESS_TASK
```

## Opcode Mapping

| flux-ese | FLUX Opcode | Hex |
|---|---|---|
| `setup: x = N` | `SETUP_CONST reg, f64` | 0xA0 |
| `read sensor` | `ENERGY_REPORT reg` | 0x60 |
| `trust_of(x)` | `TRUST_COMPARE` | 0x50 |
| `confidence.score = ... * N` | `CONF_GET → LOAD_CONST → CONF_MUL` | 0x40, 0x10, 0x42 |
| `instinct.modulate(...)` | `INST_MODULATE` | 0x70 |
| `delegate task to X` | `DELEGATE reg` | 0x80 |
| `reply "msg"` | `STORE_STRING → REPLY` | 0xF0, 0x81 |
| `process task` | `STORE_STRING → PROCESS_TASK` | 0xF0, 0x82 |
| `if x < y:` | `CMP → JLT` | 0x20, 0x21 |
| `if x > y:` | `CMP → JGT` | 0x20, 0x22 |
| `LOAD N` | `LOAD_CONST reg, f64` | 0x10 |

## Architecture

- `lexer.rs` — Tokenizer: keywords, identifiers, numbers, strings, operators
- `parser.rs` — Recursive descent parser producing AST
- `ast.rs` — AST types: FluxProgram, BlockItem, Expr, Stmt
- `compiler.rs` — AST → FLUX bytecode emitter
- `opcodes.rs` — FLUX VM opcode definitions with real hex values

## Running

```bash
cargo build
cargo test
cargo run --example guard    # parses examples/guard.fluxese
```
