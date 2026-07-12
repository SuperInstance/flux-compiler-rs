// decoder.rs — Decode bytecode bytes back into instructions and human-readable assembly.

use crate::errors::ValidationError;
use crate::parser::{self, Format, OpcodeInfo};

/// A decoded instruction from raw bytecode.
#[derive(Debug, Clone)]
pub struct DecodedInstruction {
    pub info: &'static OpcodeInfo,
    pub regs: Vec<u8>,
    pub imm: Option<i16>,
    pub addr: Option<u16>,
    pub data: Option<Vec<u8>>,
    pub offset: usize,
}

/// Decode bytecode into a vector of decoded instructions.
pub fn decode_bytecode(bytecode: &[u8]) -> Result<Vec<DecodedInstruction>, ValidationError> {
    if bytecode.is_empty() {
        return Err(ValidationError::Empty);
    }

    let mut result = Vec::new();
    let mut pc = 0usize;

    while pc < bytecode.len() {
        let opcode = bytecode[pc];
        let info = parser::lookup_opcode(opcode).ok_or(ValidationError::InvalidOpcode(opcode))?;
        let offset = pc;

        match info.format {
            Format::A => {
                result.push(DecodedInstruction {
                    info,
                    regs: vec![],
                    imm: None,
                    addr: None,
                    data: None,
                    offset,
                });
                pc += 1;
            }
            Format::B => {
                let needed = 2;
                if pc + needed > bytecode.len() {
                    return Err(ValidationError::Truncated {
                        opcode,
                        needed: needed - 1,
                        remaining: bytecode.len() - pc - 1,
                    });
                }
                let reg = bytecode[pc + 1];
                if reg > 15 {
                    return Err(ValidationError::InvalidRegisterByte { byte: reg });
                }
                result.push(DecodedInstruction {
                    info,
                    regs: vec![reg],
                    imm: None,
                    addr: None,
                    data: None,
                    offset,
                });
                pc += 2;
            }
            Format::B2 => {
                let needed = 3;
                if pc + needed > bytecode.len() {
                    return Err(ValidationError::Truncated {
                        opcode,
                        needed: needed - 1,
                        remaining: bytecode.len() - pc - 1,
                    });
                }
                let lo = bytecode[pc + 1] as u16;
                let hi = bytecode[pc + 2] as u16;
                let addr = lo | (hi << 8);
                result.push(DecodedInstruction {
                    info,
                    regs: vec![],
                    imm: None,
                    addr: Some(addr),
                    data: None,
                    offset,
                });
                pc += 3;
            }
            Format::C => {
                let needed = 3;
                if pc + needed > bytecode.len() {
                    return Err(ValidationError::Truncated {
                        opcode,
                        needed: needed - 1,
                        remaining: bytecode.len() - pc - 1,
                    });
                }
                let rd = bytecode[pc + 1];
                let rs1 = bytecode[pc + 2];
                for &r in &[rd, rs1] {
                    if r > 15 {
                        return Err(ValidationError::InvalidRegisterByte { byte: r });
                    }
                }
                result.push(DecodedInstruction {
                    info,
                    regs: vec![rd, rs1],
                    imm: None,
                    addr: None,
                    data: None,
                    offset,
                });
                pc += 3;
            }
            Format::D => {
                let needed = 4;
                if pc + needed > bytecode.len() {
                    return Err(ValidationError::Truncated {
                        opcode,
                        needed: needed - 1,
                        remaining: bytecode.len() - pc - 1,
                    });
                }
                let reg = bytecode[pc + 1];
                if reg > 15 {
                    return Err(ValidationError::InvalidRegisterByte { byte: reg });
                }
                let lo = bytecode[pc + 2] as u8;
                let hi = bytecode[pc + 3] as u8;
                let imm = i16::from_le_bytes([lo, hi]);
                result.push(DecodedInstruction {
                    info,
                    regs: vec![reg],
                    imm: Some(imm),
                    addr: None,
                    data: None,
                    offset,
                });
                pc += 4;
            }
            Format::E => {
                let needed = 4;
                if pc + needed > bytecode.len() {
                    return Err(ValidationError::Truncated {
                        opcode,
                        needed: needed - 1,
                        remaining: bytecode.len() - pc - 1,
                    });
                }
                let rd = bytecode[pc + 1];
                let rs1 = bytecode[pc + 2];
                let rs2 = bytecode[pc + 3];
                for &r in &[rd, rs1, rs2] {
                    if r > 15 {
                        return Err(ValidationError::InvalidRegisterByte { byte: r });
                    }
                }
                result.push(DecodedInstruction {
                    info,
                    regs: vec![rd, rs1, rs2],
                    imm: None,
                    addr: None,
                    data: None,
                    offset,
                });
                pc += 4;
            }
            Format::G => {
                let header = 3; // opcode + 2 len bytes
                if pc + header > bytecode.len() {
                    return Err(ValidationError::Truncated {
                        opcode,
                        needed: header - 1,
                        remaining: bytecode.len() - pc - 1,
                    });
                }
                let len_lo = bytecode[pc + 1] as usize;
                let len_hi = bytecode[pc + 2] as usize;
                let data_len = len_lo | (len_hi << 8);
                let total = header + data_len;
                if pc + total > bytecode.len() {
                    return Err(ValidationError::DataLengthOverflow {
                        declared: data_len,
                        available: bytecode.len() - pc - header,
                    });
                }
                let data = bytecode[pc + header..pc + total].to_vec();
                result.push(DecodedInstruction {
                    info,
                    regs: vec![],
                    imm: None,
                    addr: None,
                    data: Some(data),
                    offset,
                });
                pc += total;
            }
        }
    }

    Ok(result)
}

/// Disassemble bytecode into human-readable FLUX assembly text.
pub fn disassemble_bytecode(bytecode: &[u8]) -> Result<String, ValidationError> {
    let instructions = decode_bytecode(bytecode)?;
    let mut out = String::new();

    for inst in &instructions {
        // Add offset comment
        out.push_str(&format!("  ; offset 0x{:04X}\n", inst.offset));

        let line = format_instruction(inst);
        out.push_str("  ");
        out.push_str(&line);
        out.push('\n');
    }

    Ok(out)
}

/// Format a single decoded instruction as a human-readable assembly line.
fn format_instruction(inst: &DecodedInstruction) -> String {
    match inst.info.format {
        Format::A => inst.info.mnemonic.to_string(),
        Format::B => format!("{} R{}", inst.info.mnemonic, inst.regs[0]),
        Format::B2 => {
            let addr = inst.addr.unwrap_or(0);
            format!("{} 0x{:04X}", inst.info.mnemonic, addr)
        }
        Format::C => format!(
            "{} R{}, R{}",
            inst.info.mnemonic, inst.regs[0], inst.regs[1]
        ),
        Format::D => {
            let imm = inst.imm.unwrap_or(0);
            // For MOVI, show as immediate. For jumps, show as relative offset.
            if inst.info.opcode == 0x2B {
                format!("{} R{}, {}", inst.info.mnemonic, inst.regs[0], imm)
            } else {
                format!("{} R{}, {}", inst.info.mnemonic, inst.regs[0], imm)
            }
        }
        Format::E => format!(
            "{} R{}, R{}, R{}",
            inst.info.mnemonic, inst.regs[0], inst.regs[1], inst.regs[2]
        ),
        Format::G => {
            let data = inst.data.as_deref().unwrap_or(&[]);
            let hex: String = data.iter().map(|b| format!("{:02X}", b)).collect();
            format!("{} 0x{}", inst.info.mnemonic, hex)
        }
    }
}

impl DecodedInstruction {
    /// Calculate the byte length of this decoded instruction.
    pub fn byte_len(&self) -> usize {
        match self.info.format {
            Format::A => 1,
            Format::B => 2,
            Format::B2 => 3,
            Format::C => 3,
            Format::D => 4,
            Format::E => 4,
            Format::G => 3 + self.data.as_ref().map_or(0, |d| d.len()),
        }
    }
}

// Allow conversion from DecodedInstruction to String via Display.
impl std::fmt::Display for DecodedInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format_instruction(self))
    }
}

// Re-export the public decode and disassemble functions with simpler names.
pub fn disassemble(bytecode: &[u8]) -> String {
    match disassemble_bytecode(bytecode) {
        Ok(s) => s,
        Err(e) => format!("; disassembly error: {e}\n"),
    }
}

pub fn decode(bytecode: &[u8]) -> Result<Vec<DecodedInstruction>, ValidationError> {
    decode_bytecode(bytecode)
}
