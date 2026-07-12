// Integration tests for si-flux-compiler

use si_flux_compiler::{assemble, disassemble, validate, round_trip, AssemblyError, ValidationError};

// ---------------------------------------------------------------------------
// Format A — NOP, HALT, YIELD, DUP, SWAP
// ---------------------------------------------------------------------------

#[test]
fn test_nop() {
    let bc = assemble("NOP").unwrap();
    assert_eq!(bc, vec![0x00]);
}

#[test]
fn test_halt() {
    let bc = assemble("HALT").unwrap();
    assert_eq!(bc, vec![0x80]);
}

#[test]
fn test_yield() {
    let bc = assemble("YIELD").unwrap();
    assert_eq!(bc, vec![0x81]);
}

#[test]
fn test_dup() {
    let bc = assemble("DUP").unwrap();
    assert_eq!(bc, vec![0x22]);
}

#[test]
fn test_swap() {
    let bc = assemble("SWAP").unwrap();
    assert_eq!(bc, vec![0x23]);
}

#[test]
fn test_format_a_sequence() {
    let bc = assemble("NOP\nNOP\nHALT").unwrap();
    assert_eq!(bc, vec![0x00, 0x00, 0x80]);
}

// ---------------------------------------------------------------------------
// Format B — INC, DEC, PUSH, POP
// ---------------------------------------------------------------------------

#[test]
fn test_inc() {
    let bc = assemble("INC R5").unwrap();
    assert_eq!(bc, vec![0x0E, 0x05]);
}

#[test]
fn test_dec() {
    let bc = assemble("DEC R0").unwrap();
    assert_eq!(bc, vec![0x0F, 0x00]);
}

#[test]
fn test_push() {
    let bc = assemble("PUSH R15").unwrap();
    assert_eq!(bc, vec![0x20, 0x0F]);
}

#[test]
fn test_pop() {
    let bc = assemble("POP R3").unwrap();
    assert_eq!(bc, vec![0x21, 0x03]);
}

// ---------------------------------------------------------------------------
// Format B2 — JE, JNE (16-bit absolute address)
// ---------------------------------------------------------------------------

#[test]
fn test_je_absolute() {
    let bc = assemble("JE 0x0100").unwrap();
    assert_eq!(bc, vec![0x2E, 0x00, 0x01]);
}

#[test]
fn test_jne_absolute() {
    let bc = assemble("JNE 42").unwrap();
    assert_eq!(bc, vec![0x2F, 0x2A, 0x00]);
}

#[test]
fn test_je_label() {
    let source = "JE target\ntarget:\nHALT";
    let bc = assemble(source).unwrap();
    // HALT is at offset 3 (after JE's 3 bytes)
    assert_eq!(bc, vec![0x2E, 0x03, 0x00, 0x80]);
}

// ---------------------------------------------------------------------------
// Format C — MOV, LOAD, STORE, CMP, RET, INEG, INOT
// ---------------------------------------------------------------------------

#[test]
fn test_mov() {
    let bc = assemble("MOV R1, R2").unwrap();
    assert_eq!(bc, vec![0x01, 0x01, 0x02]);
}

#[test]
fn test_load() {
    let bc = assemble("LOAD R0, R5").unwrap();
    assert_eq!(bc, vec![0x02, 0x00, 0x05]);
}

#[test]
fn test_store() {
    let bc = assemble("STORE R3, R7").unwrap();
    assert_eq!(bc, vec![0x03, 0x03, 0x07]);
}

#[test]
fn test_cmp() {
    let bc = assemble("CMP R1, R2").unwrap();
    assert_eq!(bc, vec![0x2D, 0x01, 0x02]);
}

#[test]
fn test_ret() {
    let bc = assemble("RET R0, R1").unwrap();
    assert_eq!(bc, vec![0x28, 0x00, 0x01]);
}

#[test]
fn test_ineg() {
    let bc = assemble("INEG R0, R3").unwrap();
    assert_eq!(bc, vec![0x0D, 0x00, 0x03]);
}

#[test]
fn test_inot() {
    let bc = assemble("INOT R1, R2").unwrap();
    assert_eq!(bc, vec![0x13, 0x01, 0x02]);
}

// ---------------------------------------------------------------------------
// Format D — MOVI, JMP, JZ, JNZ, CALL
// ---------------------------------------------------------------------------

#[test]
fn test_movi_decimal() {
    let bc = assemble("MOVI R0, 42").unwrap();
    assert_eq!(bc, vec![0x2B, 0x00, 0x2A, 0x00]);
}

#[test]
fn test_movi_hex() {
    let bc = assemble("MOVI R1, 0xFF").unwrap();
    assert_eq!(bc, vec![0x2B, 0x01, 0xFF, 0x00]);
}

#[test]
fn test_movi_negative() {
    let bc = assemble("MOVI R2, -10").unwrap();
    // -10 as i16 LE = 0xFFF6 → bytes 0xF6, 0xFF
    assert_eq!(bc, vec![0x2B, 0x02, 0xF6, 0xFF]);
}

#[test]
fn test_movi_binary() {
    let bc = assemble("MOVI R0, 0b101010").unwrap();
    assert_eq!(bc, vec![0x2B, 0x00, 0x2A, 0x00]);
}

#[test]
fn test_jmp() {
    let bc = assemble("JMP R0, 0").unwrap();
    assert_eq!(bc, vec![0x04, 0x00, 0x00, 0x00]);
}

#[test]
fn test_jz() {
    let bc = assemble("JZ R1, -4").unwrap();
    assert_eq!(bc, vec![0x05, 0x01, 0xFC, 0xFF]);
}

#[test]
fn test_jnz() {
    let bc = assemble("JNZ R2, 100").unwrap();
    assert_eq!(bc, vec![0x06, 0x02, 0x64, 0x00]);
}

#[test]
fn test_call() {
    let bc = assemble("CALL R0, 8").unwrap();
    assert_eq!(bc, vec![0x07, 0x00, 0x08, 0x00]);
}

// ---------------------------------------------------------------------------
// Format E — IADD, ISUB, IMUL, IDIV, IMOD, IAND, IOR, IXOR, ISHL, ISHR
//           FADD, FSUB, FMUL, FDIV
// ---------------------------------------------------------------------------

#[test]
fn test_iadd() {
    let bc = assemble("IADD R0, R1, R2").unwrap();
    assert_eq!(bc, vec![0x08, 0x00, 0x01, 0x02]);
}

#[test]
fn test_isub() {
    let bc = assemble("ISUB R3, R4, R5").unwrap();
    assert_eq!(bc, vec![0x09, 0x03, 0x04, 0x05]);
}

#[test]
fn test_imul() {
    let bc = assemble("IMUL R0, R0, R1").unwrap();
    assert_eq!(bc, vec![0x0A, 0x00, 0x00, 0x01]);
}

#[test]
fn test_idiv() {
    let bc = assemble("IDIV R6, R7, R8").unwrap();
    assert_eq!(bc, vec![0x0B, 0x06, 0x07, 0x08]);
}

#[test]
fn test_imod() {
    let bc = assemble("IMOD R0, R1, R2").unwrap();
    assert_eq!(bc, vec![0x0C, 0x00, 0x01, 0x02]);
}

#[test]
fn test_iand() {
    let bc = assemble("IAND R1, R2, R3").unwrap();
    assert_eq!(bc, vec![0x10, 0x01, 0x02, 0x03]);
}

#[test]
fn test_ior() {
    let bc = assemble("IOR R1, R2, R3").unwrap();
    assert_eq!(bc, vec![0x11, 0x01, 0x02, 0x03]);
}

#[test]
fn test_ixor() {
    let bc = assemble("IXOR R4, R5, R6").unwrap();
    assert_eq!(bc, vec![0x12, 0x04, 0x05, 0x06]);
}

#[test]
fn test_ishl() {
    let bc = assemble("ISHL R0, R1, R2").unwrap();
    assert_eq!(bc, vec![0x14, 0x00, 0x01, 0x02]);
}

#[test]
fn test_ishr() {
    let bc = assemble("ISHR R0, R1, R2").unwrap();
    assert_eq!(bc, vec![0x15, 0x00, 0x01, 0x02]);
}

#[test]
fn test_fadd() {
    let bc = assemble("FADD F0, F1, F2").unwrap();
    assert_eq!(bc, vec![0x40, 0x00, 0x01, 0x02]);
}

#[test]
fn test_fsub() {
    let bc = assemble("FSUB F0, F1, F2").unwrap();
    assert_eq!(bc, vec![0x41, 0x00, 0x01, 0x02]);
}

#[test]
fn test_fmul() {
    let bc = assemble("FMUL F3, F4, F5").unwrap();
    assert_eq!(bc, vec![0x42, 0x03, 0x04, 0x05]);
}

#[test]
fn test_fdiv() {
    let bc = assemble("FDIV F0, F1, F2").unwrap();
    assert_eq!(bc, vec![0x43, 0x00, 0x01, 0x02]);
}

// ---------------------------------------------------------------------------
// Format G — TELL, ASK, DELEGATE, BROADCAST
// ---------------------------------------------------------------------------

#[test]
fn test_tell_hex() {
    let bc = assemble("TELL 0xDEADBEEF").unwrap();
    assert_eq!(bc, vec![0x60, 0x04, 0x00, 0xDE, 0xAD, 0xBE, 0xEF]);
}

#[test]
fn test_ask_string() {
    let bc = assemble("ASK \"hello\"").unwrap();
    assert_eq!(bc, vec![0x61, 0x05, 0x00, b'h', b'e', b'l', b'l', b'o']);
}

#[test]
fn test_broadcast() {
    let bc = assemble("BROADCAST 0xABCD").unwrap();
    assert_eq!(bc, vec![0x66, 0x02, 0x00, 0xAB, 0xCD]);
}

#[test]
fn test_delegate() {
    let bc = assemble("DELEGATE 0x01").unwrap();
    assert_eq!(bc, vec![0x62, 0x01, 0x00, 0x01]);
}

// ---------------------------------------------------------------------------
// Labels and jumps
// ---------------------------------------------------------------------------

#[test]
fn test_label_backward_jump() {
    let source = "loop:\nNOP\nDEC R0\nJNZ R0, loop\nHALT";
    let bc = assemble(source).unwrap();
    // Layout: NOP(1) DEC(2) JNZ(4) HALT(1) = 8 bytes
    // loop = offset 0
    // JNZ at offset 3, relative offset = 0 - (3+4) = -7
    let expected = vec![0x00, 0x0F, 0x00, 0x06, 0x00, 0xF9, 0xFF, 0x80];
    assert_eq!(bc, expected);
}

#[test]
fn test_label_forward_jump() {
    let source = "JMP R0, end\nNOP\nend:\nHALT";
    let bc = assemble(source).unwrap();
    // JMP at offset 0 (4 bytes), NOP at 4 (1 byte), HALT at 5
    // offset = 5 - (0+4) = 1
    let expected = vec![0x04, 0x00, 0x01, 0x00, 0x00, 0x80];
    assert_eq!(bc, expected);
}

#[test]
fn test_label_je_forward() {
    let source = "JE skip\nMOVI R0, 1\nskip:\nHALT";
    let bc = assemble(source).unwrap();
    // JE(3) MOVI(4) HALT(1)
    // skip = offset 7
    assert_eq!(bc, vec![0x2E, 0x07, 0x00, 0x2B, 0x00, 0x01, 0x00, 0x80]);
}

#[test]
fn test_multiple_labels() {
    let source = "
        MOVI R0, 0
    loop:
        INC R0
        CMP R0, R1
        JNE loop
        HALT
    ";
    let bc = assemble(source).unwrap();
    // MOVI(4) at 0, loop at 4, INC(2) at 4, CMP(3) at 6, JNE(3) at 9, HALT(1) at 12
    // JNE target = loop = 4 → absolute 0x04
    assert_eq!(bc, vec![0x2B, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x2D, 0x00, 0x01, 0x2F, 0x04, 0x00, 0x80]);
}

#[test]
fn test_unknown_label_errors() {
    let err = assemble("JMP R0, nowhere").unwrap_err();
    assert!(matches!(err, AssemblyError::UnknownLabel(_)));
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

#[test]
fn test_semicolon_comment() {
    let bc = assemble("NOP ; this is a comment\nHALT").unwrap();
    assert_eq!(bc, vec![0x00, 0x80]);
}

#[test]
fn test_slash_comment() {
    let bc = assemble("NOP // comment\nHALT").unwrap();
    assert_eq!(bc, vec![0x00, 0x80]);
}

#[test]
fn test_inline_comment_after_operand() {
    let bc = assemble("MOVI R0, 42 ; set counter").unwrap();
    assert_eq!(bc, vec![0x2B, 0x00, 0x2A, 0x00]);
}

// ---------------------------------------------------------------------------
// Case insensitivity
// ---------------------------------------------------------------------------

#[test]
fn test_lowercase_mnemonic() {
    let bc = assemble("mov r1, r2").unwrap();
    assert_eq!(bc, vec![0x01, 0x01, 0x02]);
}

#[test]
fn test_mixed_case_register() {
    let bc = assemble("INC r7").unwrap();
    assert_eq!(bc, vec![0x0E, 0x07]);
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[test]
fn test_unknown_mnemonic() {
    let err = assemble("FOO R0, R1").unwrap_err();
    assert!(matches!(err, AssemblyError::UnknownMnemonic(_)));
}

#[test]
fn test_invalid_register() {
    let err = assemble("INC R16").unwrap_err();
    assert!(matches!(err, AssemblyError::InvalidRegister(_)));
}

#[test]
fn test_too_many_operands() {
    let err = assemble("NOP R0").unwrap_err();
    assert!(matches!(err, AssemblyError::OperandCount { .. }));
}

#[test]
fn test_too_few_operands() {
    let err = assemble("MOV R0").unwrap_err();
    assert!(matches!(err, AssemblyError::OperandCount { .. }));
}

#[test]
fn test_immediate_out_of_range() {
    let err = assemble("MOVI R0, 99999").unwrap_err();
    assert!(matches!(err, AssemblyError::ValueOutOfRange { .. }));
}

// ---------------------------------------------------------------------------
// Validate
// ---------------------------------------------------------------------------

#[test]
fn test_validate_empty() {
    assert_eq!(validate(&[]), Err(ValidationError::Empty));
}

#[test]
fn test_validate_valid() {
    let bc = assemble("NOP\nHALT").unwrap();
    validate(&bc).unwrap();
}

#[test]
fn test_validate_truncated_b() {
    // INC opcode with missing register byte
    assert_eq!(validate(&[0x0E]), Err(ValidationError::Truncated { opcode: 0x0E, needed: 1, remaining: 0 }));
}

#[test]
fn test_validate_truncated_d() {
    // MOVI opcode with only 2 bytes instead of 4
    assert_eq!(validate(&[0x2B, 0x00, 0x2A]), Err(ValidationError::Truncated { opcode: 0x2B, needed: 3, remaining: 2 }));
}

#[test]
fn test_validate_invalid_opcode() {
    assert_eq!(validate(&[0xFF]), Err(ValidationError::InvalidOpcode(0xFF)));
}

#[test]
fn test_validate_invalid_register_byte() {
    // MOV with register byte 0x10 (>15)
    assert_eq!(validate(&[0x01, 0x10, 0x00]), Err(ValidationError::InvalidRegisterByte { byte: 0x10 }));
}

#[test]
fn test_validate_truncated_g() {
    // TELL with length prefix saying 4 bytes but only 2 remaining
    assert_eq!(
        validate(&[0x60, 0x04, 0x00, 0xDE, 0xAD]),
        Err(ValidationError::DataLengthOverflow { declared: 4, available: 2 })
    );
}

// ---------------------------------------------------------------------------
// Disassemble
// ---------------------------------------------------------------------------

#[test]
fn test_disassemble_nop() {
    let asm = disassemble(&[0x00]);
    assert!(asm.contains("NOP"));
}

#[test]
fn test_disassemble_movi() {
    let asm = disassemble(&[0x2B, 0x00, 0x2A, 0x00]);
    assert!(asm.contains("MOVI"));
    assert!(asm.contains("R0"));
    assert!(asm.contains("42"));
}

#[test]
fn test_disassemble_iadd() {
    let asm = disassemble(&[0x08, 0x01, 0x02, 0x03]);
    assert!(asm.contains("IADD"));
    assert!(asm.contains("R1"));
    assert!(asm.contains("R2"));
    assert!(asm.contains("R3"));
}

#[test]
fn test_disassemble_format_g() {
    let asm = disassemble(&[0x60, 0x02, 0x00, 0xDE, 0xAD]);
    assert!(asm.contains("TELL"));
    assert!(asm.contains("DEAD"));
}

#[test]
fn test_disassemble_je() {
    let asm = disassemble(&[0x2E, 0x10, 0x00]);
    assert!(asm.contains("JE"));
    assert!(asm.contains("0x0010"));
}

// ---------------------------------------------------------------------------
// Round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_round_trip_basic() {
    let source = "NOP\nMOVI R0, 42\nIADD R0, R0, R1\nHALT";
    let rt = round_trip(source).unwrap();
    assert!(rt.contains("NOP"));
    assert!(rt.contains("MOVI"));
    assert!(rt.contains("IADD"));
    assert!(rt.contains("HALT"));
}

#[test]
fn test_round_trip_all_formats() {
    let source = "
        NOP
        INC R0
        JE 10
        MOV R1, R2
        MOVI R3, -5
        IADD R4, R5, R6
        TELL 0xABCD
        HALT
    ";
    let bc = assemble(source).unwrap();
    validate(&bc).unwrap();
    let asm = disassemble(&bc);
    assert!(asm.contains("NOP"));
    assert!(asm.contains("INC"));
    assert!(asm.contains("JE"));
    assert!(asm.contains("MOV"));
    assert!(asm.contains("MOVI"));
    assert!(asm.contains("IADD"));
    assert!(asm.contains("TELL"));
    assert!(asm.contains("HALT"));
}

// ---------------------------------------------------------------------------
// Complex program
// ---------------------------------------------------------------------------

#[test]
fn test_complex_program() {
    // Compute sum 1..5 in R0
    let source = "
        ; Compute sum 1..5
        MOVI R0, 0       ; accumulator
        MOVI R1, 1       ; counter
        MOVI R2, 5       ; limit
    loop:
        IADD R0, R0, R1  ; acc += counter
        INC R1           ; counter++
        CMP R1, R2       ; compare counter to limit
        JNE loop         ; keep going if counter != limit
        HALT
    ";
    let bc = assemble(source).unwrap();
    validate(&bc).unwrap();

    // Verify bytecode structure
    // MOVI R0,0 (4) MOVI R1,1 (4) MOVI R2,5 (4) = 12 bytes at offset 0,4,8
    // loop at offset 12
    // IADD (4) at 12, INC (2) at 16, CMP (3) at 18, JNE (3) at 21, HALT (1) at 24
    // JNE target = loop = 12 → absolute 0x0C
    assert_eq!(bc.len(), 25);
    assert_eq!(bc[12], 0x08); // IADD
    assert_eq!(bc[21], 0x2F); // JNE
    assert_eq!(bc[22], 0x0C); // addr lo = 12
    assert_eq!(bc[23], 0x00); // addr hi = 0
    assert_eq!(bc[24], 0x80); // HALT
}

#[test]
fn test_stack_operations() {
    let source = "
        MOVI R0, 10
        PUSH R0
        MOVI R0, 20
        PUSH R0
        POP R1
        POP R2
        HALT
    ";
    let bc = assemble(source).unwrap();
    validate(&bc).unwrap();
    // 6 instructions: 4+2+4+2+2+2+1 = 17 bytes
    assert_eq!(bc.len(), 17);
}

#[test]
fn test_float_operations() {
    let source = "
        FADD F0, F1, F2
        FSUB F3, F4, F5
        FMUL F0, F0, F1
        FDIV F2, F3, F4
        FNEG F0, F1
        HALT
    ";
    let bc = assemble(source).unwrap();
    validate(&bc).unwrap();
    // FADD(4) FSUB(4) FMUL(4) FDIV(4) FNEG(3) HALT(1) = 20
    assert_eq!(bc.len(), 20);
}

#[test]
fn test_optional_commas() {
    // Without commas
    let bc1 = assemble("MOV R1 R2").unwrap();
    // With commas
    let bc2 = assemble("MOV R1, R2").unwrap();
    assert_eq!(bc1, bc2);
}

#[test]
fn test_blank_lines_and_whitespace() {
    let source = "

        NOP

        HALT

    ";
    let bc = assemble(source).unwrap();
    assert_eq!(bc, vec![0x00, 0x80]);
}

#[test]
fn test_label_on_same_line_as_instruction() {
    let source = "loop: NOP\nJMP R0, loop\nHALT";
    let bc = assemble(source).unwrap();
    // NOP(1) at 0, loop at 0
    // JMP(4) at 1, offset = 0 - (1+4) = -5
    // HALT(1) at 5
    assert_eq!(bc, vec![0x00, 0x04, 0x00, 0xFB, 0xFF, 0x80]);
}

// ---------------------------------------------------------------------------
// Additional Python-only opcodes (parseable)
// ---------------------------------------------------------------------------

#[test]
fn test_ieq() {
    let bc = assemble("IEQ R0, R1, R2").unwrap();
    assert_eq!(bc, vec![0x19, 0x00, 0x01, 0x02]);
}

#[test]
fn test_rot() {
    let bc = assemble("ROT").unwrap();
    assert_eq!(bc, vec![0x24]);
}

#[test]
fn test_enter_leave() {
    let bc = assemble("ENTER R0, 8\nLEAVE R0, 4").unwrap();
    assert_eq!(bc, vec![0x25, 0x00, 0x08, 0x00, 0x26, 0x00, 0x04, 0x00]);
}

#[test]
fn test_barrier() {
    let bc = assemble("BARRIER").unwrap();
    assert_eq!(bc, vec![0x78]);
}

#[test]
fn test_simd() {
    let bc = assemble("VADD V0, V1, V2").unwrap();
    assert_eq!(bc, vec![0x52, 0x00, 0x01, 0x02]);
}
