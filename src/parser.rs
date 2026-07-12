// parser.rs — Parse FLUX assembly text into an instruction AST.

use crate::errors::AssemblyError;
use std::collections::HashMap;

/// The seven FLUX instruction formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    A,  // opcode only           — 1 byte
    B,  // opcode + reg          — 2 bytes
    B2, // opcode + u16 addr     — 3 bytes
    C,  // opcode + 2 regs       — 3 bytes
    D,  // opcode + reg + i16    — 4 bytes
    E,  // opcode + 3 regs       — 4 bytes
    G,  // opcode + u16 len + data — variable
}

/// Static info for a FLUX opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpcodeInfo {
    pub opcode: u8,
    pub mnemonic: &'static str,
    pub format: Format,
}

/// All recognised FLUX opcodes.
pub static OPCODE_TABLE: &[OpcodeInfo] = &[
    // Core
    OpcodeInfo { opcode: 0x00, mnemonic: "NOP",       format: Format::A },
    OpcodeInfo { opcode: 0x01, mnemonic: "MOV",       format: Format::C },
    OpcodeInfo { opcode: 0x02, mnemonic: "LOAD",      format: Format::C },
    OpcodeInfo { opcode: 0x03, mnemonic: "STORE",     format: Format::C },
    OpcodeInfo { opcode: 0x04, mnemonic: "JMP",       format: Format::D },
    OpcodeInfo { opcode: 0x05, mnemonic: "JZ",        format: Format::D },
    OpcodeInfo { opcode: 0x06, mnemonic: "JNZ",       format: Format::D },
    OpcodeInfo { opcode: 0x07, mnemonic: "CALL",      format: Format::D },
    OpcodeInfo { opcode: 0x08, mnemonic: "IADD",      format: Format::E },
    OpcodeInfo { opcode: 0x09, mnemonic: "ISUB",      format: Format::E },
    OpcodeInfo { opcode: 0x0A, mnemonic: "IMUL",      format: Format::E },
    OpcodeInfo { opcode: 0x0B, mnemonic: "IDIV",      format: Format::E },
    OpcodeInfo { opcode: 0x0C, mnemonic: "IMOD",      format: Format::E },
    OpcodeInfo { opcode: 0x0D, mnemonic: "INEG",      format: Format::C },
    OpcodeInfo { opcode: 0x0E, mnemonic: "INC",       format: Format::B },
    OpcodeInfo { opcode: 0x0F, mnemonic: "DEC",       format: Format::B },
    OpcodeInfo { opcode: 0x10, mnemonic: "IAND",      format: Format::E },
    OpcodeInfo { opcode: 0x11, mnemonic: "IOR",       format: Format::E },
    OpcodeInfo { opcode: 0x12, mnemonic: "IXOR",      format: Format::E },
    OpcodeInfo { opcode: 0x13, mnemonic: "INOT",      format: Format::C },
    OpcodeInfo { opcode: 0x14, mnemonic: "ISHL",      format: Format::E },
    OpcodeInfo { opcode: 0x15, mnemonic: "ISHR",      format: Format::E },
    // Extended comparison
    OpcodeInfo { opcode: 0x19, mnemonic: "IEQ",       format: Format::E },
    OpcodeInfo { opcode: 0x1A, mnemonic: "ILT",       format: Format::E },
    OpcodeInfo { opcode: 0x1B, mnemonic: "ILE",       format: Format::E },
    OpcodeInfo { opcode: 0x1C, mnemonic: "IGT",       format: Format::E },
    OpcodeInfo { opcode: 0x1D, mnemonic: "IGE",       format: Format::E },
    OpcodeInfo { opcode: 0x1E, mnemonic: "TEST",      format: Format::C },
    OpcodeInfo { opcode: 0x1F, mnemonic: "SETCC",     format: Format::C },
    // Stack
    OpcodeInfo { opcode: 0x20, mnemonic: "PUSH",      format: Format::B },
    OpcodeInfo { opcode: 0x21, mnemonic: "POP",       format: Format::B },
    OpcodeInfo { opcode: 0x22, mnemonic: "DUP",       format: Format::A },
    OpcodeInfo { opcode: 0x23, mnemonic: "SWAP",      format: Format::A },
    OpcodeInfo { opcode: 0x24, mnemonic: "ROT",       format: Format::A },
    OpcodeInfo { opcode: 0x25, mnemonic: "ENTER",     format: Format::D },
    OpcodeInfo { opcode: 0x26, mnemonic: "LEAVE",     format: Format::D },
    OpcodeInfo { opcode: 0x27, mnemonic: "ALLOCA",    format: Format::D },
    OpcodeInfo { opcode: 0x28, mnemonic: "RET",       format: Format::C },
    OpcodeInfo { opcode: 0x2B, mnemonic: "MOVI",      format: Format::D },
    OpcodeInfo { opcode: 0x2D, mnemonic: "CMP",       format: Format::C },
    OpcodeInfo { opcode: 0x2E, mnemonic: "JE",        format: Format::B2 },
    OpcodeInfo { opcode: 0x2F, mnemonic: "JNE",       format: Format::B2 },
    // Memory region ops
    OpcodeInfo { opcode: 0x30, mnemonic: "REGION_CREATE",  format: Format::G },
    OpcodeInfo { opcode: 0x31, mnemonic: "REGION_DESTROY", format: Format::G },
    OpcodeInfo { opcode: 0x32, mnemonic: "REGION_TRANSFER",format: Format::G },
    OpcodeInfo { opcode: 0x33, mnemonic: "MEMCOPY",   format: Format::G },
    OpcodeInfo { opcode: 0x34, mnemonic: "MEMSET",    format: Format::G },
    OpcodeInfo { opcode: 0x35, mnemonic: "MEMCMP",    format: Format::G },
    // Additional jumps
    OpcodeInfo { opcode: 0x36, mnemonic: "JL",        format: Format::D },
    OpcodeInfo { opcode: 0x37, mnemonic: "JGE",       format: Format::D },
    // Type ops
    OpcodeInfo { opcode: 0x38, mnemonic: "CAST",      format: Format::C },
    OpcodeInfo { opcode: 0x39, mnemonic: "BOX",       format: Format::C },
    OpcodeInfo { opcode: 0x3A, mnemonic: "UNBOX",     format: Format::C },
    OpcodeInfo { opcode: 0x3B, mnemonic: "CHECK_TYPE",    format: Format::G },
    OpcodeInfo { opcode: 0x3C, mnemonic: "CHECK_BOUNDS",  format: Format::G },
    // Meta
    OpcodeInfo { opcode: 0x3D, mnemonic: "CONF",      format: Format::G },
    OpcodeInfo { opcode: 0x3E, mnemonic: "MERGE",     format: Format::G },
    OpcodeInfo { opcode: 0x3F, mnemonic: "RESTORE",   format: Format::G },
    // Float ops
    OpcodeInfo { opcode: 0x40, mnemonic: "FADD",      format: Format::E },
    OpcodeInfo { opcode: 0x41, mnemonic: "FSUB",      format: Format::E },
    OpcodeInfo { opcode: 0x42, mnemonic: "FMUL",      format: Format::E },
    OpcodeInfo { opcode: 0x43, mnemonic: "FDIV",      format: Format::E },
    OpcodeInfo { opcode: 0x44, mnemonic: "FNEG",      format: Format::C },
    OpcodeInfo { opcode: 0x45, mnemonic: "FABS",      format: Format::C },
    OpcodeInfo { opcode: 0x46, mnemonic: "FMIN",      format: Format::E },
    OpcodeInfo { opcode: 0x47, mnemonic: "FMAX",      format: Format::E },
    OpcodeInfo { opcode: 0x48, mnemonic: "FEQ",       format: Format::E },
    OpcodeInfo { opcode: 0x49, mnemonic: "FNE",       format: Format::E },
    OpcodeInfo { opcode: 0x4A, mnemonic: "FLT",       format: Format::E },
    OpcodeInfo { opcode: 0x4B, mnemonic: "FLE",       format: Format::E },
    OpcodeInfo { opcode: 0x4C, mnemonic: "FGT",       format: Format::E },
    OpcodeInfo { opcode: 0x4D, mnemonic: "FGE",       format: Format::E },
    OpcodeInfo { opcode: 0x4E, mnemonic: "JLE",       format: Format::D },
    OpcodeInfo { opcode: 0x4F, mnemonic: "JG",        format: Format::D },
    // SIMD
    OpcodeInfo { opcode: 0x50, mnemonic: "VLOAD",     format: Format::E },
    OpcodeInfo { opcode: 0x51, mnemonic: "VSTORE",    format: Format::E },
    OpcodeInfo { opcode: 0x52, mnemonic: "VADD",      format: Format::E },
    OpcodeInfo { opcode: 0x53, mnemonic: "VSUB",      format: Format::E },
    OpcodeInfo { opcode: 0x54, mnemonic: "VMUL",      format: Format::E },
    OpcodeInfo { opcode: 0x55, mnemonic: "VDIV",      format: Format::E },
    OpcodeInfo { opcode: 0x56, mnemonic: "VFMA",      format: Format::E },
    // A2A
    OpcodeInfo { opcode: 0x60, mnemonic: "TELL",      format: Format::G },
    OpcodeInfo { opcode: 0x61, mnemonic: "ASK",       format: Format::G },
    OpcodeInfo { opcode: 0x62, mnemonic: "DELEGATE",  format: Format::G },
    OpcodeInfo { opcode: 0x66, mnemonic: "BROADCAST", format: Format::G },
    // Trust/capability
    OpcodeInfo { opcode: 0x70, mnemonic: "TRUST_SET",     format: Format::G },
    OpcodeInfo { opcode: 0x71, mnemonic: "TRUST_GET",     format: Format::G },
    OpcodeInfo { opcode: 0x72, mnemonic: "TRUST_REVOKE",  format: Format::G },
    OpcodeInfo { opcode: 0x73, mnemonic: "TRUST_LIST",    format: Format::G },
    OpcodeInfo { opcode: 0x74, mnemonic: "CAP_GRANT",     format: Format::G },
    OpcodeInfo { opcode: 0x75, mnemonic: "CAP_REVOKE",    format: Format::G },
    OpcodeInfo { opcode: 0x76, mnemonic: "CAP_CHECK",     format: Format::G },
    OpcodeInfo { opcode: 0x77, mnemonic: "CAP_LIST",      format: Format::G },
    OpcodeInfo { opcode: 0x78, mnemonic: "BARRIER",       format: Format::A },
    OpcodeInfo { opcode: 0x79, mnemonic: "SYNC_CLOCK",    format: Format::A },
    // Evolution
    OpcodeInfo { opcode: 0x7C, mnemonic: "EVOLVE",   format: Format::G },
    OpcodeInfo { opcode: 0x7D, mnemonic: "INSTINCT", format: Format::G },
    OpcodeInfo { opcode: 0x7E, mnemonic: "WITNESS",  format: Format::G },
    OpcodeInfo { opcode: 0x7F, mnemonic: "SNAPSHOT", format: Format::G },
    // Control
    OpcodeInfo { opcode: 0x80, mnemonic: "HALT",      format: Format::A },
    OpcodeInfo { opcode: 0x81, mnemonic: "YIELD",     format: Format::A },
    // Marine physics
    OpcodeInfo { opcode: 0xB0, mnemonic: "PHY_CREATE",    format: Format::G },
    OpcodeInfo { opcode: 0xB1, mnemonic: "PHY_STEP",      format: Format::G },
    OpcodeInfo { opcode: 0xB2, mnemonic: "PHY_APPLY",     format: Format::G },
    OpcodeInfo { opcode: 0xB3, mnemonic: "PHY_QUERY",     format: Format::G },
    OpcodeInfo { opcode: 0xB4, mnemonic: "PHY_DESTROY",   format: Format::G },
    OpcodeInfo { opcode: 0xB5, mnemonic: "PHY_SYNC",      format: Format::G },
    OpcodeInfo { opcode: 0xB6, mnemonic: "PHY_MERGE",     format: Format::G },
    OpcodeInfo { opcode: 0xB7, mnemonic: "PHY_SPLIT",     format: Format::G },
    OpcodeInfo { opcode: 0xB8, mnemonic: "PHY_STATUS",    format: Format::G },
];

/// A parsed instruction ready for encoding.
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub info: &'static OpcodeInfo,
    pub regs: Vec<u8>,
    pub imm: Option<i16>,
    pub addr: Option<u16>,
    pub data: Option<Vec<u8>>,
    pub line: usize,
    pub offset: usize,
}

impl Instruction {
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

/// Look up an opcode by mnemonic (case-insensitive).
pub fn lookup_mnemonic(mnemonic: &str) -> Option<&'static OpcodeInfo> {
    let upper = mnemonic.to_ascii_uppercase();
    OPCODE_TABLE.iter().find(|o| o.mnemonic == upper)
}

/// Look up an opcode by byte value.
pub fn lookup_opcode(byte: u8) -> Option<&'static OpcodeInfo> {
    OPCODE_TABLE.iter().find(|o| o.opcode == byte)
}

// ---------------------------------------------------------------------------
// Text parsing
// ---------------------------------------------------------------------------

/// Result of parsing — instructions with all labels resolved.
pub struct ParseResult {
    pub instructions: Vec<Instruction>,
    pub labels: HashMap<String, usize>,
}

/// Parse FLUX assembly source into an instruction list, resolving all labels.
pub fn parse(source: &str) -> Result<ParseResult, AssemblyError> {
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut labels: HashMap<String, usize> = HashMap::new();
    let mut current_offset: usize = 0;

    for (line_idx, raw_line) in source.lines().enumerate() {
        let line_no = line_idx + 1;
        let line = strip_comment(raw_line).trim();

        if line.is_empty() {
            continue;
        }

        // Handle `label:` possibly followed by an instruction.
        let (label_part, rest) = match line.split_once(':') {
            Some((lbl, rest)) => {
                let lbl = lbl.trim();
                validate_label_name(lbl, line_no)?;
                (Some(lbl.to_string()), rest.trim())
            }
            None => (None, line),
        };

        if let Some(lbl) = label_part {
            if labels.contains_key(&lbl) {
                return Err(AssemblyError::SyntaxError(format!(
                    "line {line_no}: duplicate label '{lbl}'"
                )));
            }
            labels.insert(lbl, current_offset);
        }

        if rest.is_empty() {
            continue;
        }

        let inst = parse_instruction(rest, line_no, current_offset)?;
        current_offset += inst.byte_len();
        instructions.push(inst);
    }

    resolve_labels(&mut instructions, &labels)?;
    Ok(ParseResult { instructions, labels })
}

fn validate_label_name(lbl: &str, line_no: usize) -> Result<(), AssemblyError> {
    if lbl.is_empty() {
        return Err(AssemblyError::SyntaxError(format!(
            "line {line_no}: empty label name"
        )));
    }
    let first = lbl.chars().next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return Err(AssemblyError::SyntaxError(format!(
            "line {line_no}: invalid label name '{lbl}'"
        )));
    }
    if !lbl.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(AssemblyError::SyntaxError(format!(
            "line {line_no}: invalid label name '{lbl}'"
        )));
    }
    Ok(())
}

fn strip_comment(line: &str) -> &str {
    let semi = line.find(';');
    let slash = line.find("//");
    match (semi, slash) {
        (Some(s), Some(c)) => &line[..s.min(c)],
        (Some(s), None) => &line[..s],
        (None, Some(c)) => &line[..c],
        (None, None) => line,
    }
}

fn parse_instruction(text: &str, line_no: usize, offset: usize) -> Result<Instruction, AssemblyError> {
    let mut parts = text.split_whitespace();
    let mnemonic = parts.next().ok_or_else(|| {
        AssemblyError::SyntaxError(format!("line {line_no}: empty instruction"))
    })?;

    let info = lookup_mnemonic(mnemonic).ok_or_else(|| {
        AssemblyError::UnknownMnemonic(format!("'{mnemonic}' (line {line_no})"))
    })?;

    let operand_strs: Vec<String> = parts
        .flat_map(|p| {
            p.split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect::<Vec<_>>()
        })
        .collect();

    let expected = match info.format {
        Format::A => 0,
        Format::B => 1,
        Format::B2 => 1,
        Format::C => 2,
        Format::D => 2,
        Format::E => 3,
        Format::G => 1,
    };

    if operand_strs.len() != expected {
        return Err(AssemblyError::OperandCount {
            mnemonic: info.mnemonic.to_string(),
            expected,
            got: operand_strs.len(),
        });
    }

    let mut inst = Instruction {
        info,
        regs: Vec::new(),
        imm: None,
        addr: None,
        data: None,
        line: line_no,
        offset,
    };

    match info.format {
        Format::A => {}
        Format::B => {
            inst.regs.push(parse_register(&operand_strs[0], line_no)?);
        }
        Format::B2 => {
            if let Some(addr) = parse_u16_or_label(&operand_strs[0])? {
                inst.addr = Some(addr);
            } else {
                inst.data = Some(operand_strs[0].as_bytes().to_vec());
            }
        }
        Format::C => {
            inst.regs.push(parse_register(&operand_strs[0], line_no)?);
            inst.regs.push(parse_register(&operand_strs[1], line_no)?);
        }
        Format::D => {
            inst.regs.push(parse_register(&operand_strs[0], line_no)?);
            if let Some(val) = parse_i16_or_label(&operand_strs[1])? {
                inst.imm = Some(val);
            } else {
                inst.data = Some(operand_strs[1].as_bytes().to_vec());
            }
        }
        Format::E => {
            inst.regs.push(parse_register(&operand_strs[0], line_no)?);
            inst.regs.push(parse_register(&operand_strs[1], line_no)?);
            inst.regs.push(parse_register(&operand_strs[2], line_no)?);
        }
        Format::G => {
            let data = parse_data_payload(&operand_strs[0], line_no)?;
            inst.data = Some(data);
        }
    }

    Ok(inst)
}

/// Parse a register token: R0–R15, F0–F15, V0–V15 (case-insensitive).
pub fn parse_register(tok: &str, line_no: usize) -> Result<u8, AssemblyError> {
    let tok = tok.trim();
    let upper = tok.to_ascii_uppercase();

    let num_str = if upper.starts_with('R') || upper.starts_with('F') || upper.starts_with('V') {
        &upper[1..]
    } else {
        return Err(AssemblyError::InvalidRegister(format!(
            "'{tok}' (line {line_no}) — expected R0–R15 / F0–F15"
        )));
    };

    let n: u8 = num_str.parse().map_err(|_| {
        AssemblyError::InvalidRegister(format!("'{tok}' (line {line_no})"))
    })?;

    if n > 15 {
        return Err(AssemblyError::InvalidRegister(format!(
            "'{tok}' (line {line_no}) — register index must be 0–15"
        )));
    }

    Ok(n)
}

fn parse_i16_or_label(tok: &str) -> Result<Option<i16>, AssemblyError> {
    let tok = tok.trim();
    if tok
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or(false)
    {
        return Ok(None);
    }
    let val = parse_integer(tok)?;
    if val < i16::MIN as i64 || val > i16::MAX as i64 {
        return Err(AssemblyError::ValueOutOfRange {
            value: val,
            min: i16::MIN as i64,
            max: i16::MAX as i64,
        });
    }
    Ok(Some(val as i16))
}

fn parse_u16_or_label(tok: &str) -> Result<Option<u16>, AssemblyError> {
    let tok = tok.trim();
    if tok
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or(false)
    {
        return Ok(None);
    }
    let val = parse_integer(tok)?;
    if val < 0 || val > u16::MAX as i64 {
        return Err(AssemblyError::ValueOutOfRange {
            value: val,
            min: 0,
            max: u16::MAX as i64,
        });
    }
    Ok(Some(val as u16))
}

fn parse_integer(tok: &str) -> Result<i64, AssemblyError> {
    let tok = tok.trim().trim_end_matches(',');

    if let Some(hex) = tok.strip_prefix("0x").or_else(|| tok.strip_prefix("0X")) {
        i64::from_str_radix(hex, 16)
            .map_err(|_| AssemblyError::InvalidImmediate(format!("'{tok}'")))
    } else if let Some(bin) = tok.strip_prefix("0b").or_else(|| tok.strip_prefix("0B")) {
        i64::from_str_radix(bin, 2)
            .map_err(|_| AssemblyError::InvalidImmediate(format!("'{tok}'")))
    } else if let Some(neg) = tok.strip_prefix('-') {
        let v: i64 = neg
            .parse()
            .map_err(|_| AssemblyError::InvalidImmediate(format!("'{tok}'")))?;
        Ok(-v)
    } else {
        tok.parse::<i64>()
            .map_err(|_| AssemblyError::InvalidImmediate(format!("'{tok}'")))
    }
}

fn parse_data_payload(tok: &str, line_no: usize) -> Result<Vec<u8>, AssemblyError> {
    let tok = tok.trim();

    // Quoted string
    if (tok.starts_with('"') && tok.ends_with('"') && tok.len() >= 2)
        || (tok.starts_with('\'') && tok.ends_with('\'') && tok.len() >= 2)
    {
        let inner = &tok[1..tok.len() - 1];
        return Ok(inner.as_bytes().to_vec());
    }

    let hex = if let Some(h) = tok.strip_prefix("0x").or_else(|| tok.strip_prefix("0X")) {
        h
    } else {
        tok
    };

    if hex.len() % 2 != 0 {
        return Err(AssemblyError::InvalidImmediate(format!(
            "'{tok}' (line {line_no}) — hex payload must have even length"
        )));
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for chunk in hex.as_bytes().chunks(2) {
        let s = std::str::from_utf8(chunk)
            .map_err(|_| AssemblyError::InvalidImmediate(format!("'{tok}' (line {line_no})")))?;
        let b = u8::from_str_radix(s, 16)
            .map_err(|_| AssemblyError::InvalidImmediate(format!("'{tok}' (line {line_no})")))?;
        bytes.push(b);
    }

    Ok(bytes)
}

fn resolve_labels(
    instructions: &mut [Instruction],
    labels: &HashMap<String, usize>,
) -> Result<(), AssemblyError> {
    for inst in instructions.iter_mut() {
        match inst.info.format {
            Format::D if inst.data.is_some() => {
                let label_bytes = inst.data.take().unwrap();
                let label_name = String::from_utf8(label_bytes)
                    .map_err(|_| AssemblyError::Parse("invalid UTF-8 in label".into()))?;
                let label_name = label_name.trim().to_string();

                let target = *labels.get(&label_name).ok_or_else(|| {
                    AssemblyError::UnknownLabel(label_name.clone())
                })?;

                if inst.info.opcode == 0x2B {
                    // MOVI — absolute address as immediate
                    inst.imm = Some(target as i16);
                } else {
                    // Relative offset: target - (inst.offset + 4)
                    let rel = target as i64 - (inst.offset as i64 + 4);
                    if rel < i16::MIN as i64 || rel > i16::MAX as i64 {
                        return Err(AssemblyError::ValueOutOfRange {
                            value: rel,
                            min: i16::MIN as i64,
                            max: i16::MAX as i64,
                        });
                    }
                    inst.imm = Some(rel as i16);
                }
            }
            Format::B2 if inst.data.is_some() => {
                let label_bytes = inst.data.take().unwrap();
                let label_name = String::from_utf8(label_bytes)
                    .map_err(|_| AssemblyError::Parse("invalid UTF-8 in label".into()))?;
                let label_name = label_name.trim().to_string();

                let target = *labels.get(&label_name).ok_or_else(|| {
                    AssemblyError::UnknownLabel(label_name.clone())
                })?;

                inst.addr = Some(target as u16);
                inst.data = None;
            }
            _ => {}
        }
    }

    Ok(())
}
