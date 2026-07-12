use std::fmt;

/// Errors encountered during assembly (source text → bytecode).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssemblyError {
    /// The mnemonic is not a recognised FLUX opcode.
    UnknownMnemonic(String),
    /// The operand count does not match what the opcode expects.
    OperandCount { mnemonic: String, expected: usize, got: usize },
    /// A register token is outside R0–R15 / F0–F15.
    InvalidRegister(String),
    /// An integer literal could not be parsed.
    InvalidImmediate(String),
    /// A label reference could not be resolved.
    UnknownLabel(String),
    /// A numeric value does not fit the target width.
    ValueOutOfRange { value: i64, min: i64, max: i64 },
    /// A line was syntactically invalid.
    SyntaxError(String),
    /// Generic I/O or parse error.
    Parse(String),
}

impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownMnemonic(m) => write!(f, "unknown mnemonic: {m}"),
            Self::OperandCount { mnemonic, expected, got } => write!(
                f,
                "{mnemonic} expects {expected} operand(s), got {got}"
            ),
            Self::InvalidRegister(r) => write!(f, "invalid register: {r}"),
            Self::InvalidImmediate(i) => write!(f, "invalid immediate: {i}"),
            Self::UnknownLabel(l) => write!(f, "unknown label: {l}"),
            Self::ValueOutOfRange { value, min, max } => {
                write!(f, "value {value} out of range [{min}, {max}]")
            }
            Self::SyntaxError(s) => write!(f, "syntax error: {s}"),
            Self::Parse(s) => write!(f, "parse error: {s}"),
        }
    }
}

impl std::error::Error for AssemblyError {}

/// Errors encountered during bytecode validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// The byte stream ended mid-instruction.
    Truncated { opcode: u8, needed: usize, remaining: usize },
    /// An opcode byte is not a valid FLUX instruction.
    InvalidOpcode(u8),
    /// A register byte is > 15.
    InvalidRegisterByte { byte: u8 },
    /// A Format-G length prefix exceeds the remaining bytes.
    DataLengthOverflow { declared: usize, available: usize },
    /// The bytecode is empty.
    Empty,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { opcode, needed, remaining } => write!(
                f,
                "truncated instruction: opcode 0x{opcode:02X} needs {needed} more byte(s), only {remaining} remaining"
            ),
            Self::InvalidOpcode(b) => write!(f, "invalid opcode: 0x{b:02X}"),
            Self::InvalidRegisterByte { byte } => {
                write!(f, "register byte out of range: 0x{byte:02X} (> 0x0F)")
            }
            Self::DataLengthOverflow { declared, available } => write!(
                f,
                "Format-G data length {declared} exceeds remaining bytes ({available})"
            ),
            Self::Empty => write!(f, "bytecode is empty"),
        }
    }
}

impl std::error::Error for ValidationError {}
