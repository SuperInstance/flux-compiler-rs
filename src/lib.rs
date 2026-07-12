//! # si-flux-compiler
//!
//! FLUX bytecode assembler, disassembler, and validator.
//!
//! This crate implements the FLUX (Fluid Language Universal eXecution) bytecode
//! format v1.1 as defined in [FLUX_BYTECODE_SPEC.md](https://github.com/SuperInstance/AI-Writings/blob/main/FLUX_BYTECODE_SPEC.md).
//!
//! ## Quick Start
//!
//! ```rust
//! use si_flux_compiler::{assemble, disassemble, validate};
//!
//! let source = "MOVI R0, 42\nIADD R0, R0, R1\nHALT";
//! let bytecode = assemble(source).unwrap();
//! validate(&bytecode).unwrap();
//! let asm = disassemble(&bytecode);
//! println!("{asm}");
//! ```
//!
//! ## Instruction Formats
//!
//! | Format | Size    | Layout                         | Example Instructions        |
//! |--------|---------|--------------------------------|-----------------------------|
//! | A      | 1 byte  | opcode                         | NOP, HALT, YIELD            |
//! | B      | 2 bytes | opcode + reg                   | INC, DEC, PUSH              |
//! | B₂     | 3 bytes | opcode + u16 addr (LE)         | JE, JNE                     |
//! | C      | 3 bytes | opcode + 2 regs                | MOV, LOAD, STORE, CMP       |
//! | D      | 4 bytes | opcode + reg + i16 imm (LE)    | MOVI, JMP, JZ, JNZ, CALL    |
//! | E      | 4 bytes | opcode + 3 regs                | IADD, ISUB, IMUL, FADD      |
//! | G      | var     | opcode + u16 len (LE) + data   | TELL, ASK, DELEGATE, BROADCAST |
//!
//! ## WASM
//!
//! Enable the `wasm` feature for `wasm-bindgen` exports.

pub mod errors;
pub mod parser;
pub mod encoder;
pub mod decoder;

pub use errors::{AssemblyError, ValidationError};
pub use parser::{Format, Instruction, OpcodeInfo, OPCODE_TABLE, lookup_mnemonic, lookup_opcode};
pub use encoder::{assemble_source, encode, encode_instruction};
pub use decoder::{decode, decode_bytecode, disassemble_bytecode, DecodedInstruction};

/// Assemble FLUX assembly source text into bytecode.
///
/// ```rust
/// # use si_flux_compiler::assemble;
/// let bc = assemble("MOVI R0, 10\nHALT").unwrap();
/// assert_eq!(&bc[..3], &[0x2B, 0x00, 0x0A]);
/// ```
pub fn assemble(source: &str) -> Result<Vec<u8>, AssemblyError> {
    assemble_source(source)
}

/// Disassemble bytecode into human-readable FLUX assembly.
///
/// ```rust
/// # use si_flux_compiler::disassemble;
/// let asm = disassemble(&[0x00, 0x80]); // NOP; HALT
/// assert!(asm.contains("NOP"));
/// assert!(asm.contains("HALT"));
/// ```
pub fn disassemble(bytecode: &[u8]) -> String {
    decoder::disassemble(bytecode)
}

// re-export from decoder

/// Validate that a bytecode buffer is well-formed.
///
/// Checks:
/// - Non-empty
/// - All opcodes recognised
/// - No truncated instructions
/// - Register bytes ≤ 0x0F
/// - Format-G length prefixes don't overflow
///
/// ```rust
/// # use si_flux_compiler::validate;
/// validate(&[0x00, 0x80]).unwrap(); // NOP; HALT — OK
/// validate(&[]).unwrap_err();       // empty — error
/// ```
pub fn validate(bytecode: &[u8]) -> Result<(), ValidationError> {
    // decode_bytecode performs all structural validation.
    decoder::decode_bytecode(bytecode)?;
    Ok(())
}

/// Round-trip: assemble then disassemble. Useful for testing and pretty-printing.
pub fn round_trip(source: &str) -> Result<String, AssemblyError> {
    let bc = assemble(source)?;
    Ok(disassemble(&bc))
}

// ---------------------------------------------------------------------------
// WASM bindings
// ---------------------------------------------------------------------------

#[cfg(feature = "wasm")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn assemble(source: &str) -> Result<Vec<u8>, JsValue> {
        crate::assemble(source).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn disassemble(bytecode: &[u8]) -> String {
        crate::disassemble(bytecode)
    }

    #[wasm_bindgen]
    pub fn validate(bytecode: &[u8]) -> Result<(), JsValue> {
        crate::validate(bytecode).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
