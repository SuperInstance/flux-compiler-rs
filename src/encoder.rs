// encoder.rs — Encode parsed instructions to bytecode bytes.

use crate::errors::AssemblyError;
use crate::parser::{self, Format, Instruction, ParseResult};

/// Encode a full parse result into bytecode bytes.
pub fn encode(parsed: &ParseResult) -> Result<Vec<u8>, AssemblyError> {
    let mut out = Vec::with_capacity(parsed.instructions.iter().map(|i| i.byte_len()).sum());
    for inst in &parsed.instructions {
        encode_instruction(inst, &mut out)?;
    }
    Ok(out)
}

/// Encode a single instruction, appending bytes to `out`.
pub fn encode_instruction(inst: &Instruction, out: &mut Vec<u8>) -> Result<(), AssemblyError> {
    match inst.info.format {
        Format::A => {
            out.push(inst.info.opcode);
        }
        Format::B => {
            out.push(inst.info.opcode);
            out.push(inst.regs[0]);
        }
        Format::B2 => {
            out.push(inst.info.opcode);
            let addr = inst.addr.ok_or_else(|| {
                AssemblyError::Parse(format!(
                    "B2 instruction {} at line {} missing address",
                    inst.info.mnemonic, inst.line
                ))
            })?;
            out.push((addr & 0xFF) as u8); // lo
            out.push((addr >> 8) as u8);   // hi
        }
        Format::C => {
            out.push(inst.info.opcode);
            out.push(inst.regs[0]);
            out.push(inst.regs[1]);
        }
        Format::D => {
            out.push(inst.info.opcode);
            out.push(inst.regs[0]);
            let imm = inst.imm.ok_or_else(|| {
                AssemblyError::Parse(format!(
                    "D instruction {} at line {} missing immediate",
                    inst.info.mnemonic, inst.line
                ))
            })?;
            out.push((imm & 0xFF) as u8); // lo byte
            out.push(((imm >> 8) & 0xFF) as u8); // hi byte
        }
        Format::E => {
            out.push(inst.info.opcode);
            out.push(inst.regs[0]);
            out.push(inst.regs[1]);
            out.push(inst.regs[2]);
        }
        Format::G => {
            out.push(inst.info.opcode);
            let data = inst.data.as_ref().ok_or_else(|| {
                AssemblyError::Parse(format!(
                    "G instruction {} at line {} missing data payload",
                    inst.info.mnemonic, inst.line
                ))
            })?;
            let len = data.len();
            if len > u16::MAX as usize {
                return Err(AssemblyError::ValueOutOfRange {
                    value: len as i64,
                    min: 0,
                    max: u16::MAX as i64,
                });
            }
            out.push((len & 0xFF) as u8); // len_lo
            out.push((len >> 8) as u8);   // len_hi
            out.extend_from_slice(data);
        }
    }
    Ok(())
}

/// Convenience: assemble source text directly into bytecode.
pub fn assemble_source(source: &str) -> Result<Vec<u8>, AssemblyError> {
    let parsed = parser::parse(source)?;
    encode(&parsed)
}
