# 🔧 FLUX Compiler (Rust)

![Crates.io](https://img.shields.io/crates/v/si-flux-compiler)
![Rust](https://img.shields.io/badge/rust-stable-orange)
![WASM](https://img.shields.io/badge/wasm-ready-654ff0)
![Tests](https://img.shields.io/badge/tests-60%2B-brightgreen)
![License](https://img.shields.io/badge/License-MIT-yellow)

**FLUX bytecode assembler, disassembler, and validator.**

Implements the [FLUX Bytecode Format Specification v1.1](https://github.com/SuperInstance/AI-Writings/blob/main/FLUX_BYTECODE_SPEC.md). Compiles FLUX assembly text to bytecode, disassembles bytecode back to human-readable assembly, and validates structural integrity. WASM-ready for browser use.

---

## Philosophy

Part of [Working Animal Architecture](https://github.com/SuperInstance/AI-Writings), where **γ + η = C** (genome + nurture = capability). The compiler is the **forge** — it shapes the steel of FLUX bytecode from raw assembly text. The bytecode it produces is the fence that governs working animals. A well-forged fence is invisible to the animal and impenetrable to chaos.

> *The compiler doesn't interpret policy. It forge-shapes it.*

## Features

- **Assemble** — FLUX assembly text → bytecode (`assemble`)
- **Disassemble** — bytecode → human-readable assembly (`disassemble`)
- **Validate** — structural integrity checks (`validate`)
- **Round-trip** — assemble then disassemble for pretty-printing (`round_trip`)
- **WASM-ready** — compile to `wasm32-unknown-unknown` for browser use

## Installation

```bash
cargo add si-flux-compiler
```

Or in `Cargo.toml`:

```toml
[dependencies]
si-flux-compiler = "0.1"
```

For WASM builds:

```toml
[dependencies]
si-flux-compiler = { version = "0.1", features = ["wasm"] }
```

## Quick Start

### Assemble, validate, disassemble

```rust
use si_flux_compiler::{assemble, disassemble, validate};

fn main() {
    let source = r#"
        MOVI R0, 0
        MOVI R1, 10
    loop:
        IADD R0, R0, R1
        DEC R1
        JNZ R1, loop
        HALT
    "#;

    let bytecode = assemble(source).unwrap();
    validate(&bytecode).unwrap();
    println!("{}", disassemble(&bytecode));
}
```

### Round-trip for pretty-printing

```rust
use si_flux_compiler::round_trip;

// Assemble messy source, disassemble to clean output
let clean = round_trip("MOVI R0, 42\nIADD R0,R0,R1\nHALT").unwrap();
println!("{clean}");
// NOP
// MOVI R0, 0x2A
// IADD R0, R0, R1
// HALT
```

### Hex and binary immediates

```rust
use si_flux_compiler::assemble;

let bc = assemble("MOVI R0, 0xFF\nMOVI R1, 0b1010\nHALT").unwrap();
```

## Assembly Syntax

```flux
; Comments use ; or //
label_name:
    MOVI R0, 42        ; decimal immediate
    MOVI R1, 0xFF      ; hex immediate
    MOVI R2, 0b1010    ; binary immediate
    MOVI R3, -10       ; negative immediate
    IADD R0, R0, R1    ; R0 = R0 + R1
    JNZ R3, label_name ; jump if R3 != 0
    HALT
```

### Registers

| Register Banks | Range | Use |
|---------------|-------|-----|
| `R0`–`R15` | Integer | General-purpose 32-bit integer registers |
| `F0`–`F15` | Float | Floating-point registers |
| `V0`–`V15` | Vector | Vector registers (format G) |

Register names are case-insensitive (`r0` works the same as `R0`).

### Immediate Formats

| Format | Example | Value |
|--------|---------|-------|
| Decimal | `42` | 42 |
| Hexadecimal | `0xFF` | 255 |
| Binary | `0b1010` | 10 |
| Negative | `-10` | -10 (stored as i16) |

### Labels

Labels are defined with `name:` on their own line or inline before an instruction. Forward and backward references are supported. The assembler resolves labels to relative offsets at emit time.

### Commas

Commas between operands are optional: `IADD R0 R1 R2` and `IADD R0, R1, R2` are equivalent.

## Instruction Formats

| Format | Size | Layout | Instructions |
|--------|------|--------|-------------|
| A | 1 byte | `opcode` | NOP, HALT, YIELD, DUP, SWAP |
| B | 2 bytes | `opcode reg` | INC, DEC, PUSH, POP |
| B₂ | 3 bytes | `opcode addr16[LE]` | JE, JNE |
| C | 3 bytes | `opcode rd rs1` | MOV, LOAD, STORE, CMP, RET, INEG |
| D | 4 bytes | `opcode reg imm16[LE]` | MOVI, JMP, JZ, JNZ, CALL |
| E | 4 bytes | `opcode rd rs1 rs2` | IADD, ISUB, IMUL, FADD, FSUB, ... |
| G | var | `opcode len16[LE] data[len]` | TELL, ASK, DELEGATE, BROADCAST |

## API Reference

### Top-level Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `assemble(source)` | `&str → Result<Vec<u8>, AssemblyError>` | Assemble FLUX assembly to bytecode |
| `disassemble(bytecode)` | `&[u8] → String` | Disassemble bytecode to assembly text |
| `validate(bytecode)` | `&[u8] → Result<(), ValidationError>` | Validate structural integrity |
| `round_trip(source)` | `&str → Result<String, AssemblyError>` | Assemble + disassemble (pretty-print) |

### `parser` Module

| Type | Description |
|------|-------------|
| `Instruction` | Parsed instruction with format, opcode, operands |
| `Format` | Enum: A, B, B2, C, D, E, G |
| `OpcodeInfo` | Opcode metadata (mnemonic, format, size) |
| `OPCODE_TABLE` | Static lookup table for all opcodes |
| `lookup_mnemonic(byte)` | Byte → mnemonic string |
| `lookup_opcode(mnem)` | Mnemonic string → opcode byte |

### `encoder` Module

| Function | Description |
|--------|-------------|
| `assemble_source(source)` | Full assembly pipeline (parse → resolve → encode) |
| `encode(instructions)` | Encode parsed instructions to bytecode |
| `encode_instruction(instr)` | Encode a single instruction |

### `decoder` Module

| Type/Function | Description |
|------|-------------|
| `DecodedInstruction` | A decoded instruction with mnemonic and operands |
| `decode(bytecode)` | Decode bytecode into instruction list |
| `decode_bytecode(bytecode)` | Full decode with validation |
| `disassemble(bytecode)` | Decode + format as assembly text |

### Error Types

| Error | When |
|-------|------|
| `AssemblyError` | Parse failure, undefined label, bad register, bad immediate |
| `ValidationError` | Empty bytecode, unknown opcode, truncated instruction, register overflow |

## WASM Build

For browser-based FLUX compilation (e.g., in the [FLUX Visual Editor](https://github.com/SuperInstance)):

```sh
rustup target add wasm32-unknown-unknown
cargo build --features wasm --target wasm32-unknown-unknown
```

Exposes `assemble`, `disassemble`, and `validate` via `wasm-bindgen` for JavaScript consumption:

```js
import { assemble, disassemble, validate } from './si_flux_compiler.js';

const bytecode = assemble("MOVI R0, 42\nHALT");
validate(bytecode);
const text = disassemble(bytecode);
```

## Architecture

```
src/
├── lib.rs       # Public API: assemble(), disassemble(), validate(), round_trip()
├── errors.rs    # AssemblyError, ValidationError types
├── parser.rs    # Lexer + parser: source text → Instruction list
│                # Opcode table, Format enum, register/immediate parsing
├── encoder.rs   # Instruction list → bytecode (label resolution + emission)
└── decoder.rs   # Bytecode → DecodedInstruction list (for disassembly)
```

### Assembly Pipeline

```
Source text
    │
    ▼
  Parser ──── tokens → Instruction list (Format, Opcode, operands)
    │
    ▼
  Encoder ──── label resolution → offset computation → byte emission
    │
    ▼
  Bytecode (Vec<u8>)
```

## Testing

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run round-trip tests (assemble → disassemble → verify)
cargo test round_trip

# Pretty-print assertion diffs
cargo test -- --nocapture --test-threads=1
```

Uses `pretty_assertions` for readable diff output on test failures.

## Cross-Implementation

| Aspect | Python | Rust |
|--------|--------|------|
| Package | N/A (part of flux-vm) | `cargo add si-flux-compiler` |
| Repo | [flux-vm](https://github.com/SuperInstance/flux-vm) (assembler module) | [flux-compiler-rs](https://github.com/SuperInstance/flux-compiler-rs) (this) |
| WASM | N/A | ✅ (`wasm-bindgen` feature) |
| Bytecode compat | ✅ v1.1 spec | ✅ v1.1 spec |

Both implementations produce binary-compatible bytecode per the [FLUX Bytecode Format Specification v1.1](https://github.com/SuperInstance/AI-Writings/blob/main/FLUX_BYTECODE_SPEC.md).

## Ecosystem

### FLUX Runtime
- [flux-vm](https://github.com/SuperInstance/flux-vm) — Python VM with built-in assembler
- [flux-core](https://github.com/SuperInstance/flux-core) — Rust VM (`cargo add fluxvm`)
- [conservation-enforcer-rs](https://github.com/SuperInstance/conservation-enforcer-rs) — Conservation enforcement with built-in assembler

### Policy Infrastructure
- [flux-registry-rs](https://github.com/SuperInstance/flux-registry-rs) — Policy registry CLI
- [flux-policy-tester-rs](https://github.com/SuperInstance/flux-policy-tester-rs) — Policy testing framework with built-in assembler

### Theory
- [FLUX Bytecode Spec](https://github.com/SuperInstance/AI-Writings/blob/main/FLUX_BYTECODE_SPEC.md) — Formal specification
- [AI-Writings](https://github.com/SuperInstance/AI-Writings) — Paradigm essays

## License

MIT
