# si-flux-compiler

FLUX bytecode assembler, disassembler, and validator for the [FLUX Visual Editor](https://github.com/SuperInstance) backend.

Implements the [FLUX Bytecode Format Specification v1.1](https://github.com/SuperInstance/AI-Writings/blob/main/FLUX_BYTECODE_SPEC.md).

## Features

- **Assemble** FLUX assembly text → bytecode (`assemble`)
- **Disassemble** bytecode → human-readable assembly (`disassemble`)
- **Validate** bytecode is well-formed (`validate`)
- **WASM-ready** — compile to `wasm32-unknown-unknown` for browser use

## Installation

```toml
[dependencies]
si-flux-compiler = "0.1"
```

## Quick Start

```rust
use si_flux_compiler::{assemble, disassemble, validate};

fn main() {
    let source = "
        MOVI R0, 0
        MOVI R1, 10
    loop:
        IADD R0, R0, R1
        DEC R1
        JNZ R1, loop
        HALT
    ";

    let bytecode = assemble(source).unwrap();
    validate(&bytecode).unwrap();
    println!("{}", disassemble(&bytecode));
}
```

## Instruction Formats

| Format | Size    | Layout                          | Instructions                          |
|--------|---------|---------------------------------|---------------------------------------|
| A      | 1 byte  | `opcode`                        | NOP, HALT, YIELD, DUP, SWAP           |
| B      | 2 bytes | `opcode reg`                    | INC, DEC, PUSH, POP                   |
| B₂     | 3 bytes | `opcode addr16[LE]`            | JE, JNE                               |
| C      | 3 bytes | `opcode rd rs1`                 | MOV, LOAD, STORE, CMP, RET, INEG      |
| D      | 4 bytes | `opcode reg imm16[LE]`         | MOVI, JMP, JZ, JNZ, CALL             |
| E      | 4 bytes | `opcode rd rs1 rs2`            | IADD, ISUB, IMUL, FADD, FSUB, ...     |
| G      | var     | `opcode len16[LE] data[len]`   | TELL, ASK, DELEGATE, BROADCAST        |

## Assembly Syntax

```flux
; Comments use ; or //
label_name:
    MOVI R0, 42        ; decimal immediate
    MOVI R1, 0xFF      ; hex immediate
    MOVI R2, 0b1010    ; binary immediate
    MOVI R3, -10      ; negative immediate
    IADD R0, R0, R1    ; R0 = R0 + R1
    JNZ R3, label_name ; jump if R3 != 0
    HALT
```

- Registers: `R0`–`R15`, `F0`–`F15`, `V0`–`V15` (case-insensitive)
- Immediates: decimal (`42`), hex (`0x2A`), binary (`0b101010`)
- Labels: `label:` on its own line or inline before an instruction
- Commas: optional between operands

## WASM Build

```sh
rustup target add wasm32-unknown-unknown
cargo build --features wasm --target wasm32-unknown-unknown
```

Exposes `assemble`, `disassemble`, and `validate` via `wasm-bindgen`.

## Testing

```sh
cargo test
```

## License

MIT
