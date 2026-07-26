/*****************************
 * Documentation : 
 * https://www.masswerk.at/6502/6502_instruction_set.html
 * https://www.nesdev.org/wiki/Instruction_reference
 *****************************/

// ============================================================================
// ENUMS
// ============================================================================

use crate::bus::Bus;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL,
    BRK, BVC, BVS, CLC, CLD, CLI, CLV, CMP, CPX, CPY,
    DEC, DEX, DEY, EOR, INC, INX, INY, JMP, JSR, LDA,
    LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL,
    ROR, RTI, RTS, SBC, SEC, SED, SEI, STA, STX, STY,
    TAX, TAY, TSX, TXA, TXS, TYA,
}

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    Implicit,       // (INX, CLC, RTS...)
    Accumulator,    // Use A register (ASL A, ROL A...)
    Immediate,      // #$xx
    ZeroPage,       // $xx
    ZeroPageX,      // $xx,X
    ZeroPageY,      // $xx,Y
    Absolute,       // $xxxx
    AbsoluteX,      // $xxxx,X
    AbsoluteY,      // $xxxx,Y
    IndirectX,      // ($xx,X)
    IndirectY,      // ($xx),Y
    Relative,       // Branches
    Indirect,       // JMP ($xxxx)
}

#[derive(Debug, Clone, Copy)]
pub enum StatusRegister {
    C, // Carry
    Z, // Zero
    I, // Interrupt
    D, // Decimal
    B, // Break
    U, // Unused
    V, // Overflow
    N, // Negative
}

// ============================================================================
// STATUS REGISTER
// ============================================================================

impl StatusRegister {
    fn mask(self) -> u8 {
        match self {
            StatusRegister::C => 1 << 0,
            StatusRegister::Z => 1 << 1,
            StatusRegister::I => 1 << 2,
            StatusRegister::D => 1 << 3,
            StatusRegister::B => 1 << 4,
            StatusRegister::U => 1 << 5,
            StatusRegister::V => 1 << 6,
            StatusRegister::N => 1 << 7,
        }
    }
}

// ============================================================================
// STRUCTURES
// ============================================================================

pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub sr: u8,
    pub cycle: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct OpcodeInfo {
    pub instruction: Instruction,
    pub mode: AddressingMode,
    pub cycles: u8,
    pub bytes: u8,
}

// ============================================================================
// 256 OPCODES Matrix (CONST)
// ============================================================================

pub const OPCODE_TABLE: [OpcodeInfo; 256] = [
    // 0x00 - 0x0F
    OpcodeInfo { instruction: Instruction::BRK, mode: AddressingMode::Implicit,     cycles: 7, bytes: 1 }, // 0x00
    OpcodeInfo { instruction: Instruction::ORA, mode: AddressingMode::IndirectX,    cycles: 6, bytes: 2 }, // 0x01
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x02 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x03 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x04 (illégal)
    OpcodeInfo { instruction: Instruction::ORA, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0x05
    OpcodeInfo { instruction: Instruction::ASL, mode: AddressingMode::ZeroPage,     cycles: 5, bytes: 2 }, // 0x06
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x07 (illégal)
    OpcodeInfo { instruction: Instruction::PHP, mode: AddressingMode::Implicit,     cycles: 3, bytes: 1 }, // 0x08
    OpcodeInfo { instruction: Instruction::ORA, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0x09
    OpcodeInfo { instruction: Instruction::ASL, mode: AddressingMode::Accumulator,  cycles: 2, bytes: 1 }, // 0x0A
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x0B (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x0C (illégal)
    OpcodeInfo { instruction: Instruction::ORA, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0x0D
    OpcodeInfo { instruction: Instruction::ASL, mode: AddressingMode::Absolute,     cycles: 6, bytes: 3 }, // 0x0E
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x0F (illégal)
    
    // 0x10 - 0x1F
    OpcodeInfo { instruction: Instruction::BPL, mode: AddressingMode::Relative,     cycles: 2, bytes: 2 }, // 0x10 (*)
    OpcodeInfo { instruction: Instruction::ORA, mode: AddressingMode::IndirectY,    cycles: 5, bytes: 2 }, // 0x11 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x12 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x13 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x14 (illégal)
    OpcodeInfo { instruction: Instruction::ORA, mode: AddressingMode::ZeroPageX,    cycles: 4, bytes: 2 }, // 0x15
    OpcodeInfo { instruction: Instruction::ASL, mode: AddressingMode::ZeroPageX,    cycles: 6, bytes: 2 }, // 0x16
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x17 (illégal)
    OpcodeInfo { instruction: Instruction::CLC, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x18
    OpcodeInfo { instruction: Instruction::ORA, mode: AddressingMode::AbsoluteY,    cycles: 4, bytes: 3 }, // 0x19 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x1A (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x1B (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x1C (illégal)
    OpcodeInfo { instruction: Instruction::ORA, mode: AddressingMode::AbsoluteX,    cycles: 4, bytes: 3 }, // 0x1D (*)
    OpcodeInfo { instruction: Instruction::ASL, mode: AddressingMode::AbsoluteX,    cycles: 7, bytes: 3 }, // 0x1E
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x1F (illégal)
    
    // 0x20 - 0x2F
    OpcodeInfo { instruction: Instruction::JSR, mode: AddressingMode::Absolute,     cycles: 6, bytes: 3 }, // 0x20
    OpcodeInfo { instruction: Instruction::AND, mode: AddressingMode::IndirectX,    cycles: 6, bytes: 2 }, // 0x21
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x22 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x23 (illégal)
    OpcodeInfo { instruction: Instruction::BIT, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0x24
    OpcodeInfo { instruction: Instruction::AND, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0x25
    OpcodeInfo { instruction: Instruction::ROL, mode: AddressingMode::ZeroPage,     cycles: 5, bytes: 2 }, // 0x26
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x27 (illégal)
    OpcodeInfo { instruction: Instruction::PLP, mode: AddressingMode::Implicit,     cycles: 4, bytes: 1 }, // 0x28
    OpcodeInfo { instruction: Instruction::AND, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0x29
    OpcodeInfo { instruction: Instruction::ROL, mode: AddressingMode::Accumulator,  cycles: 2, bytes: 1 }, // 0x2A
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x2B (illégal)
    OpcodeInfo { instruction: Instruction::BIT, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0x2C
    OpcodeInfo { instruction: Instruction::AND, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0x2D
    OpcodeInfo { instruction: Instruction::ROL, mode: AddressingMode::Absolute,     cycles: 6, bytes: 3 }, // 0x2E
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x2F (illégal)
    
    // 0x30 - 0x3F
    OpcodeInfo { instruction: Instruction::BMI, mode: AddressingMode::Relative,     cycles: 2, bytes: 2 }, // 0x30 (*)
    OpcodeInfo { instruction: Instruction::AND, mode: AddressingMode::IndirectY,    cycles: 5, bytes: 2 }, // 0x31 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x32 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x33 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x34 (illégal)
    OpcodeInfo { instruction: Instruction::AND, mode: AddressingMode::ZeroPageX,    cycles: 4, bytes: 2 }, // 0x35
    OpcodeInfo { instruction: Instruction::ROL, mode: AddressingMode::ZeroPageX,    cycles: 6, bytes: 2 }, // 0x36
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x37 (illégal)
    OpcodeInfo { instruction: Instruction::SEC, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x38
    OpcodeInfo { instruction: Instruction::AND, mode: AddressingMode::AbsoluteY,    cycles: 4, bytes: 3 }, // 0x39 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x3A (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x3B (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x3C (illégal)
    OpcodeInfo { instruction: Instruction::AND, mode: AddressingMode::AbsoluteX,    cycles: 4, bytes: 3 }, // 0x3D (*)
    OpcodeInfo { instruction: Instruction::ROL, mode: AddressingMode::AbsoluteX,    cycles: 7, bytes: 3 }, // 0x3E
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x3F (illégal)
    
    // 0x40 - 0x4F
    OpcodeInfo { instruction: Instruction::RTI, mode: AddressingMode::Implicit,     cycles: 6, bytes: 1 }, // 0x40
    OpcodeInfo { instruction: Instruction::EOR, mode: AddressingMode::IndirectX,    cycles: 6, bytes: 2 }, // 0x41
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x42 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x43 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x44 (illégal)
    OpcodeInfo { instruction: Instruction::EOR, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0x45
    OpcodeInfo { instruction: Instruction::LSR, mode: AddressingMode::ZeroPage,     cycles: 5, bytes: 2 }, // 0x46
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x47 (illégal)
    OpcodeInfo { instruction: Instruction::PHA, mode: AddressingMode::Implicit,     cycles: 3, bytes: 1 }, // 0x48
    OpcodeInfo { instruction: Instruction::EOR, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0x49
    OpcodeInfo { instruction: Instruction::LSR, mode: AddressingMode::Accumulator,  cycles: 2, bytes: 1 }, // 0x4A
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x4B (illégal)
    OpcodeInfo { instruction: Instruction::JMP, mode: AddressingMode::Absolute,     cycles: 3, bytes: 3 }, // 0x4C
    OpcodeInfo { instruction: Instruction::EOR, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0x4D
    OpcodeInfo { instruction: Instruction::LSR, mode: AddressingMode::Absolute,     cycles: 6, bytes: 3 }, // 0x4E
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x4F (illégal)
    
    // 0x50 - 0x5F
    OpcodeInfo { instruction: Instruction::BVC, mode: AddressingMode::Relative,     cycles: 2, bytes: 2 }, // 0x50 (*)
    OpcodeInfo { instruction: Instruction::EOR, mode: AddressingMode::IndirectY,    cycles: 5, bytes: 2 }, // 0x51 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x52 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x53 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x54 (illégal)
    OpcodeInfo { instruction: Instruction::EOR, mode: AddressingMode::ZeroPageX,    cycles: 4, bytes: 2 }, // 0x55
    OpcodeInfo { instruction: Instruction::LSR, mode: AddressingMode::ZeroPageX,    cycles: 6, bytes: 2 }, // 0x56
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x57 (illégal)
    OpcodeInfo { instruction: Instruction::CLI, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x58
    OpcodeInfo { instruction: Instruction::EOR, mode: AddressingMode::AbsoluteY,    cycles: 4, bytes: 3 }, // 0x59 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x5A (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x5B (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x5C (illégal)
    OpcodeInfo { instruction: Instruction::EOR, mode: AddressingMode::AbsoluteX,    cycles: 4, bytes: 3 }, // 0x5D (*)
    OpcodeInfo { instruction: Instruction::LSR, mode: AddressingMode::AbsoluteX,    cycles: 7, bytes: 3 }, // 0x5E
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x5F (illégal)
    
    // 0x60 - 0x6F
    OpcodeInfo { instruction: Instruction::RTS, mode: AddressingMode::Implicit,     cycles: 6, bytes: 1 }, // 0x60
    OpcodeInfo { instruction: Instruction::ADC, mode: AddressingMode::IndirectX,    cycles: 6, bytes: 2 }, // 0x61
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x62 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x63 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x64 (illégal)
    OpcodeInfo { instruction: Instruction::ADC, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0x65
    OpcodeInfo { instruction: Instruction::ROR, mode: AddressingMode::ZeroPage,     cycles: 5, bytes: 2 }, // 0x66
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x67 (illégal)
    OpcodeInfo { instruction: Instruction::PLA, mode: AddressingMode::Implicit,     cycles: 4, bytes: 1 }, // 0x68
    OpcodeInfo { instruction: Instruction::ADC, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0x69
    OpcodeInfo { instruction: Instruction::ROR, mode: AddressingMode::Accumulator,  cycles: 2, bytes: 1 }, // 0x6A
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x6B (illégal)
    OpcodeInfo { instruction: Instruction::JMP, mode: AddressingMode::Indirect,     cycles: 5, bytes: 3 }, // 0x6C
    OpcodeInfo { instruction: Instruction::ADC, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0x6D
    OpcodeInfo { instruction: Instruction::ROR, mode: AddressingMode::Absolute,     cycles: 6, bytes: 3 }, // 0x6E
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x6F (illégal)
    
    // 0x70 - 0x7F
    OpcodeInfo { instruction: Instruction::BVS, mode: AddressingMode::Relative,     cycles: 2, bytes: 2 }, // 0x70 (*)
    OpcodeInfo { instruction: Instruction::ADC, mode: AddressingMode::IndirectY,    cycles: 5, bytes: 2 }, // 0x71 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x72 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x73 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x74 (illégal)
    OpcodeInfo { instruction: Instruction::ADC, mode: AddressingMode::ZeroPageX,    cycles: 4, bytes: 2 }, // 0x75
    OpcodeInfo { instruction: Instruction::ROR, mode: AddressingMode::ZeroPageX,    cycles: 6, bytes: 2 }, // 0x76
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x77 (illégal)
    OpcodeInfo { instruction: Instruction::SEI, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x78
    OpcodeInfo { instruction: Instruction::ADC, mode: AddressingMode::AbsoluteY,    cycles: 4, bytes: 3 }, // 0x79 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x7A (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x7B (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x7C (illégal)
    OpcodeInfo { instruction: Instruction::ADC, mode: AddressingMode::AbsoluteX,    cycles: 4, bytes: 3 }, // 0x7D (*)
    OpcodeInfo { instruction: Instruction::ROR, mode: AddressingMode::AbsoluteX,    cycles: 7, bytes: 3 }, // 0x7E
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x7F (illégal)
    
    // 0x80 - 0x8F
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0x80 (illégal)
    OpcodeInfo { instruction: Instruction::STA, mode: AddressingMode::IndirectX,    cycles: 6, bytes: 2 }, // 0x81
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0x82 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x83 (illégal)
    OpcodeInfo { instruction: Instruction::STY, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0x84
    OpcodeInfo { instruction: Instruction::STA, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0x85
    OpcodeInfo { instruction: Instruction::STX, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0x86
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x87 (illégal)
    OpcodeInfo { instruction: Instruction::DEY, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x88
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x89 (illégal)
    OpcodeInfo { instruction: Instruction::TXA, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x8A
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x8B (illégal)
    OpcodeInfo { instruction: Instruction::STY, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0x8C
    OpcodeInfo { instruction: Instruction::STA, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0x8D
    OpcodeInfo { instruction: Instruction::STX, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0x8E
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x8F (illégal)
    
    // 0x90 - 0x9F
    OpcodeInfo { instruction: Instruction::BCC, mode: AddressingMode::Relative,     cycles: 2, bytes: 2 }, // 0x90 (*)
    OpcodeInfo { instruction: Instruction::STA, mode: AddressingMode::IndirectY,    cycles: 6, bytes: 2 }, // 0x91
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x92 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x93 (illégal)
    OpcodeInfo { instruction: Instruction::STY, mode: AddressingMode::ZeroPageX,    cycles: 4, bytes: 2 }, // 0x94
    OpcodeInfo { instruction: Instruction::STA, mode: AddressingMode::ZeroPageX,    cycles: 4, bytes: 2 }, // 0x95
    OpcodeInfo { instruction: Instruction::STX, mode: AddressingMode::ZeroPageY,    cycles: 4, bytes: 2 }, // 0x96
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x97 (illégal)
    OpcodeInfo { instruction: Instruction::TYA, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x98
    OpcodeInfo { instruction: Instruction::STA, mode: AddressingMode::AbsoluteY,    cycles: 5, bytes: 3 }, // 0x99
    OpcodeInfo { instruction: Instruction::TXS, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x9A
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x9B (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x9C (illégal)
    OpcodeInfo { instruction: Instruction::STA, mode: AddressingMode::AbsoluteX,    cycles: 5, bytes: 3 }, // 0x9D
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x9E (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0x9F (illégal)
    
    // 0xA0 - 0xAF
    OpcodeInfo { instruction: Instruction::LDY, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0xA0
    OpcodeInfo { instruction: Instruction::LDA, mode: AddressingMode::IndirectX,    cycles: 6, bytes: 2 }, // 0xA1
    OpcodeInfo { instruction: Instruction::LDX, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0xA2
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xA3 (illégal)
    OpcodeInfo { instruction: Instruction::LDY, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0xA4
    OpcodeInfo { instruction: Instruction::LDA, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0xA5
    OpcodeInfo { instruction: Instruction::LDX, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0xA6
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xA7 (illégal)
    OpcodeInfo { instruction: Instruction::TAY, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xA8
    OpcodeInfo { instruction: Instruction::LDA, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0xA9
    OpcodeInfo { instruction: Instruction::TAX, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xAA
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xAB (illégal)
    OpcodeInfo { instruction: Instruction::LDY, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0xAC
    OpcodeInfo { instruction: Instruction::LDA, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0xAD
    OpcodeInfo { instruction: Instruction::LDX, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0xAE
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xAF (illégal)
    
    // 0xB0 - 0xBF
    OpcodeInfo { instruction: Instruction::BCS, mode: AddressingMode::Relative,     cycles: 2, bytes: 2 }, // 0xB0 (*)
    OpcodeInfo { instruction: Instruction::LDA, mode: AddressingMode::IndirectY,    cycles: 5, bytes: 2 }, // 0xB1 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xB2 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xB3 (illégal)
    OpcodeInfo { instruction: Instruction::LDY, mode: AddressingMode::ZeroPageX,    cycles: 4, bytes: 2 }, // 0xB4
    OpcodeInfo { instruction: Instruction::LDA, mode: AddressingMode::ZeroPageX,    cycles: 4, bytes: 2 }, // 0xB5
    OpcodeInfo { instruction: Instruction::LDX, mode: AddressingMode::ZeroPageY,    cycles: 4, bytes: 2 }, // 0xB6
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xB7 (illégal)
    OpcodeInfo { instruction: Instruction::CLV, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xB8
    OpcodeInfo { instruction: Instruction::LDA, mode: AddressingMode::AbsoluteY,    cycles: 4, bytes: 3 }, // 0xB9 (*)
    OpcodeInfo { instruction: Instruction::TSX, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xBA
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xBB (illégal)
    OpcodeInfo { instruction: Instruction::LDY, mode: AddressingMode::AbsoluteX,    cycles: 4, bytes: 3 }, // 0xBC (*)
    OpcodeInfo { instruction: Instruction::LDA, mode: AddressingMode::AbsoluteX,    cycles: 4, bytes: 3 }, // 0xBD (*)
    OpcodeInfo { instruction: Instruction::LDX, mode: AddressingMode::AbsoluteY,    cycles: 4, bytes: 3 }, // 0xBE (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xBF (illégal)
    
    // 0xC0 - 0xCF
    OpcodeInfo { instruction: Instruction::CPY, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0xC0
    OpcodeInfo { instruction: Instruction::CMP, mode: AddressingMode::IndirectX,    cycles: 6, bytes: 2 }, // 0xC1
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0xC2 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xC3 (illégal)
    OpcodeInfo { instruction: Instruction::CPY, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0xC4
    OpcodeInfo { instruction: Instruction::CMP, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0xC5
    OpcodeInfo { instruction: Instruction::DEC, mode: AddressingMode::ZeroPage,     cycles: 5, bytes: 2 }, // 0xC6
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xC7 (illégal)
    OpcodeInfo { instruction: Instruction::INY, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xC8
    OpcodeInfo { instruction: Instruction::CMP, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0xC9
    OpcodeInfo { instruction: Instruction::DEX, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xCA
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xCB (illégal)
    OpcodeInfo { instruction: Instruction::CPY, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0xCC
    OpcodeInfo { instruction: Instruction::CMP, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0xCD
    OpcodeInfo { instruction: Instruction::DEC, mode: AddressingMode::Absolute,     cycles: 6, bytes: 3 }, // 0xCE
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xCF (illégal)
    
    // 0xD0 - 0xDF
    OpcodeInfo { instruction: Instruction::BNE, mode: AddressingMode::Relative,     cycles: 2, bytes: 2 }, // 0xD0 (*)
    OpcodeInfo { instruction: Instruction::CMP, mode: AddressingMode::IndirectY,    cycles: 5, bytes: 2 }, // 0xD1 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xD2 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xD3 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xD4 (illégal)
    OpcodeInfo { instruction: Instruction::CMP, mode: AddressingMode::ZeroPageX,    cycles: 4, bytes: 2 }, // 0xD5
    OpcodeInfo { instruction: Instruction::DEC, mode: AddressingMode::ZeroPageX,    cycles: 6, bytes: 2 }, // 0xD6
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xD7 (illégal)
    OpcodeInfo { instruction: Instruction::CLD, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xD8
    OpcodeInfo { instruction: Instruction::CMP, mode: AddressingMode::AbsoluteY,    cycles: 4, bytes: 3 }, // 0xD9 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xDA (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xDB (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xDC (illégal)
    OpcodeInfo { instruction: Instruction::CMP, mode: AddressingMode::AbsoluteX,    cycles: 4, bytes: 3 }, // 0xDD (*)
    OpcodeInfo { instruction: Instruction::DEC, mode: AddressingMode::AbsoluteX,    cycles: 7, bytes: 3 }, // 0xDE
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xDF (illégal)
    
    // 0xE0 - 0xEF
    OpcodeInfo { instruction: Instruction::CPX, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0xE0
    OpcodeInfo { instruction: Instruction::SBC, mode: AddressingMode::IndirectX,    cycles: 6, bytes: 2 }, // 0xE1
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0xE2 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xE3 (illégal)
    OpcodeInfo { instruction: Instruction::CPX, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0xE4
    OpcodeInfo { instruction: Instruction::SBC, mode: AddressingMode::ZeroPage,     cycles: 3, bytes: 2 }, // 0xE5
    OpcodeInfo { instruction: Instruction::INC, mode: AddressingMode::ZeroPage,     cycles: 5, bytes: 2 }, // 0xE6
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xE7 (illégal)
    OpcodeInfo { instruction: Instruction::INX, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xE8
    OpcodeInfo { instruction: Instruction::SBC, mode: AddressingMode::Immediate,    cycles: 2, bytes: 2 }, // 0xE9
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xEA (officiel NOP !)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xEB (illégal)
    OpcodeInfo { instruction: Instruction::CPX, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0xEC
    OpcodeInfo { instruction: Instruction::SBC, mode: AddressingMode::Absolute,     cycles: 4, bytes: 3 }, // 0xED
    OpcodeInfo { instruction: Instruction::INC, mode: AddressingMode::Absolute,     cycles: 6, bytes: 3 }, // 0xEE
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xEF (illégal)
    
    // 0xF0 - 0xFF
    OpcodeInfo { instruction: Instruction::BEQ, mode: AddressingMode::Relative,     cycles: 2, bytes: 2 }, // 0xF0 (*)
    OpcodeInfo { instruction: Instruction::SBC, mode: AddressingMode::IndirectY,    cycles: 5, bytes: 2 }, // 0xF1 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xF2 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xF3 (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xF4 (illégal)
    OpcodeInfo { instruction: Instruction::SBC, mode: AddressingMode::ZeroPageX,    cycles: 4, bytes: 2 }, // 0xF5
    OpcodeInfo { instruction: Instruction::INC, mode: AddressingMode::ZeroPageX,    cycles: 6, bytes: 2 }, // 0xF6
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xF7 (illégal)
    OpcodeInfo { instruction: Instruction::SED, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xF8
    OpcodeInfo { instruction: Instruction::SBC, mode: AddressingMode::AbsoluteY,    cycles: 4, bytes: 3 }, // 0xF9 (*)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xFA (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xFB (illégal)
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xFC (illégal)
    OpcodeInfo { instruction: Instruction::SBC, mode: AddressingMode::AbsoluteX,    cycles: 4, bytes: 3 }, // 0xFD (*)
    OpcodeInfo { instruction: Instruction::INC, mode: AddressingMode::AbsoluteX,    cycles: 7, bytes: 3 }, // 0xFE
    OpcodeInfo { instruction: Instruction::NOP, mode: AddressingMode::Implicit,     cycles: 2, bytes: 1 }, // 0xFF (illégal)
];

// ============================================================================
// 6502 CPU Implementation
// ============================================================================
impl Cpu {
    pub fn init() -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0x0000,
            sr: 0x24,
            cycle: 0,
        }
    }

    pub fn reset(&mut self, bus: &mut Bus) {
        // read reset vector 0xFFFC-0xFFFD
        let low = bus.read(0xFFFC);
        let high = bus.read(0xFFFD);
        self.pc = ((high as u16) << 8) | (low as u16);
        
        // Reset registers
        self.sp = self.sp.wrapping_sub(3);  // decrement stack by 3
        self.sr |= 0x04;  // flag I (interrupt disable) = 1
        
        self.cycle = 0;
    }

    // --- Flags ---
    
    pub fn set_status_register_flag(&mut self, reg: StatusRegister, value: bool) {
        let mask = reg.mask();
        if value {
            self.sr |= mask;
        } else {
            self.sr &= !mask;
        }
    }

    pub fn is_status_register_flag_active(&self, reg: StatusRegister) -> bool {
        let mask = reg.mask();
        (self.sr & mask) != 0
    }

    pub fn update_status_register_nz(&mut self, value: u8) {
        self.set_status_register_flag(StatusRegister::Z, value == 0);
        self.set_status_register_flag(StatusRegister::N, value & 0b10000000 != 0);
    }

    // --- Stack ---

    fn push(&mut self, bus: &mut Bus, value: u8) {
        let address = 0x0100 + self.sp as u16;
        bus.write(address, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pull(&mut self, bus: &mut Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let address = 0x0100 + self.sp as u16;

        bus.read(address)
    }

    // --- Execute ---
    
    pub fn execute(&mut self, bus: &mut Bus) {
        let opcode = bus.read(self.pc);
        self.pc += 1;
        
        let info = &OPCODE_TABLE[opcode as usize];
        
        match (info.instruction, info.mode) {
            // ADC
            (Instruction::ADC, AddressingMode::Immediate) => self.adc_immediate(bus, info),
            (Instruction::ADC, AddressingMode::ZeroPage) => self.adc_zeropage(bus, info),
            (Instruction::ADC, AddressingMode::ZeroPageX) => self.adc_zeropagex(bus, info),
            (Instruction::ADC, AddressingMode::Absolute) => self.adc_absolute(bus, info),
            (Instruction::ADC, AddressingMode::AbsoluteX) => self.adc_absolutex(bus, info),
            (Instruction::ADC, AddressingMode::AbsoluteY) => self.adc_absolutey(bus, info),
            (Instruction::ADC, AddressingMode::IndirectX) => self.adc_indirectx(bus, info),
            (Instruction::ADC, AddressingMode::IndirectY) => self.adc_indirecty(bus, info),
            
            // AND
            (Instruction::AND, AddressingMode::Immediate) => self.and_immediate(bus, info),
            (Instruction::AND, AddressingMode::ZeroPage) => self.and_zeropage(bus, info),
            (Instruction::AND, AddressingMode::ZeroPageX) => self.and_zeropagex(bus, info),
            (Instruction::AND, AddressingMode::Absolute) => self.and_absolute(bus, info),
            (Instruction::AND, AddressingMode::AbsoluteX) => self.and_absolutex(bus, info),
            (Instruction::AND, AddressingMode::AbsoluteY) => self.and_absolutey(bus, info),
            (Instruction::AND, AddressingMode::IndirectX) => self.and_indirectx(bus, info),
            (Instruction::AND, AddressingMode::IndirectY) => self.and_indirecty(bus, info),
            
            // ASL
            (Instruction::ASL, AddressingMode::Accumulator) => self.asl_accumulator(info),
            (Instruction::ASL, AddressingMode::ZeroPage) => self.asl_zeropage(bus, info),
            (Instruction::ASL, AddressingMode::ZeroPageX) => self.asl_zeropagex(bus, info),
            (Instruction::ASL, AddressingMode::Absolute) => self.asl_absolute(bus, info),
            (Instruction::ASL, AddressingMode::AbsoluteX) => self.asl_absolutex(bus, info),
            
            // BCC
            (Instruction::BCC, AddressingMode::Relative) => self.bcc_relative(bus, info),
            
            // BCS
            (Instruction::BCS, AddressingMode::Relative) => self.bcs_relative(bus, info),
            
            // BEQ
            (Instruction::BEQ, AddressingMode::Relative) => self.beq_relative(bus, info),
            
            // BIT
            (Instruction::BIT, AddressingMode::ZeroPage) => self.bit_zeropage(bus, info),
            (Instruction::BIT, AddressingMode::Absolute) => self.bit_absolute(bus, info),
            
            // BMI
            (Instruction::BMI, AddressingMode::Relative) => self.bmi_relative(bus, info),
            
            // BNE
            (Instruction::BNE, AddressingMode::Relative) => self.bne_relative(bus, info),
            
            // BPL
            (Instruction::BPL, AddressingMode::Relative) => self.bpl_relative(bus, info),
            
            // BRK
            (Instruction::BRK, AddressingMode::Implicit) => self.brk_implicit(bus, info),
            
            // BVC
            (Instruction::BVC, AddressingMode::Relative) => self.bvc_relative(bus, info),
            
            // BVS
            (Instruction::BVS, AddressingMode::Relative) => self.bvs_relative(bus, info),
            
            // CLC
            (Instruction::CLC, AddressingMode::Implicit) => self.clc_implicit(info),
            
            // CLD
            (Instruction::CLD, AddressingMode::Implicit) => self.cld_implicit(info),
            
            // CLI
            (Instruction::CLI, AddressingMode::Implicit) => self.cli_implicit(info),
            
            // CLV
            (Instruction::CLV, AddressingMode::Implicit) => self.clv_implicit(info),
            
            // CMP
            (Instruction::CMP, AddressingMode::Immediate) => self.cmp_immediate(bus, info),
            (Instruction::CMP, AddressingMode::ZeroPage) => self.cmp_zeropage(bus, info),
            (Instruction::CMP, AddressingMode::ZeroPageX) => self.cmp_zeropagex(bus, info),
            (Instruction::CMP, AddressingMode::Absolute) => self.cmp_absolute(bus, info),
            (Instruction::CMP, AddressingMode::AbsoluteX) => self.cmp_absolutex(bus, info),
            (Instruction::CMP, AddressingMode::AbsoluteY) => self.cmp_absolutey(bus, info),
            (Instruction::CMP, AddressingMode::IndirectX) => self.cmp_indirectx(bus, info),
            (Instruction::CMP, AddressingMode::IndirectY) => self.cmp_indirecty(bus, info),
            
            // CPX
            (Instruction::CPX, AddressingMode::Immediate) => self.cpx_immediate(bus, info),
            (Instruction::CPX, AddressingMode::ZeroPage) => self.cpx_zeropage(bus, info),
            (Instruction::CPX, AddressingMode::Absolute) => self.cpx_absolute(bus, info),
            
            // CPY
            (Instruction::CPY, AddressingMode::Immediate) => self.cpy_immediate(bus, info),
            (Instruction::CPY, AddressingMode::ZeroPage) => self.cpy_zeropage(bus, info),
            (Instruction::CPY, AddressingMode::Absolute) => self.cpy_absolute(bus, info),
            
            // DEC
            (Instruction::DEC, AddressingMode::ZeroPage) => self.dec_zeropage(bus, info),
            (Instruction::DEC, AddressingMode::ZeroPageX) => self.dec_zeropagex(bus, info),
            (Instruction::DEC, AddressingMode::Absolute) => self.dec_absolute(bus, info),
            (Instruction::DEC, AddressingMode::AbsoluteX) => self.dec_absolutex(bus, info),
            
            // DEX
            (Instruction::DEX, AddressingMode::Implicit) => self.dex_implicit(info),
            
            // DEY
            (Instruction::DEY, AddressingMode::Implicit) => self.dey_implicit(info),
            
            // EOR
            (Instruction::EOR, AddressingMode::Immediate) => self.eor_immediate(bus, info),
            (Instruction::EOR, AddressingMode::ZeroPage) => self.eor_zeropage(bus, info),
            (Instruction::EOR, AddressingMode::ZeroPageX) => self.eor_zeropagex(bus, info),
            (Instruction::EOR, AddressingMode::Absolute) => self.eor_absolute(bus, info),
            (Instruction::EOR, AddressingMode::AbsoluteX) => self.eor_absolutex(bus, info),
            (Instruction::EOR, AddressingMode::AbsoluteY) => self.eor_absolutey(bus, info),
            (Instruction::EOR, AddressingMode::IndirectX) => self.eor_indirectx(bus, info),
            (Instruction::EOR, AddressingMode::IndirectY) => self.eor_indirecty(bus, info),
            
            // INC
            (Instruction::INC, AddressingMode::ZeroPage) => self.inc_zeropage(bus, info),
            (Instruction::INC, AddressingMode::ZeroPageX) => self.inc_zeropagex(bus, info),
            (Instruction::INC, AddressingMode::Absolute) => self.inc_absolute(bus, info),
            (Instruction::INC, AddressingMode::AbsoluteX) => self.inc_absolutex(bus, info),
            
            // INX
            (Instruction::INX, AddressingMode::Implicit) => self.inx_implicit(info),
            
            // INY
            (Instruction::INY, AddressingMode::Implicit) => self.iny_implicit(info),
            
            // JMP
            (Instruction::JMP, AddressingMode::Absolute) => self.jmp_absolute(bus, info),
            (Instruction::JMP, AddressingMode::Indirect) => self.jmp_indirect(bus, info),
            
            // JSR
            (Instruction::JSR, AddressingMode::Absolute) => self.jsr_absolute(bus, info),
            
            // LDA
            (Instruction::LDA, AddressingMode::Immediate) => self.lda_immediate(bus, info),
            (Instruction::LDA, AddressingMode::ZeroPage) => self.lda_zeropage(bus, info),
            (Instruction::LDA, AddressingMode::ZeroPageX) => self.lda_zeropagex(bus, info),
            (Instruction::LDA, AddressingMode::Absolute) => self.lda_absolute(bus, info),
            (Instruction::LDA, AddressingMode::AbsoluteX) => self.lda_absolutex(bus, info),
            (Instruction::LDA, AddressingMode::AbsoluteY) => self.lda_absolutey(bus, info),
            (Instruction::LDA, AddressingMode::IndirectX) => self.lda_indirectx(bus, info),
            (Instruction::LDA, AddressingMode::IndirectY) => self.lda_indirecty(bus, info),
            
            // LDX
            (Instruction::LDX, AddressingMode::Immediate) => self.ldx_immediate(bus, info),
            (Instruction::LDX, AddressingMode::ZeroPage) => self.ldx_zeropage(bus, info),
            (Instruction::LDX, AddressingMode::ZeroPageY) => self.ldx_zeropagey(bus, info),
            (Instruction::LDX, AddressingMode::Absolute) => self.ldx_absolute(bus, info),
            (Instruction::LDX, AddressingMode::AbsoluteY) => self.ldx_absolutey(bus, info),
            
            // LDY
            (Instruction::LDY, AddressingMode::Immediate) => self.ldy_immediate(bus, info),
            (Instruction::LDY, AddressingMode::ZeroPage) => self.ldy_zeropage(bus, info),
            (Instruction::LDY, AddressingMode::ZeroPageX) => self.ldy_zeropagex(bus, info),
            (Instruction::LDY, AddressingMode::Absolute) => self.ldy_absolute(bus, info),
            (Instruction::LDY, AddressingMode::AbsoluteX) => self.ldy_absolutex(bus, info),
            
            // LSR
            (Instruction::LSR, AddressingMode::Accumulator) => self.lsr_accumulator(info),
            (Instruction::LSR, AddressingMode::ZeroPage) => self.lsr_zeropage(bus, info),
            (Instruction::LSR, AddressingMode::ZeroPageX) => self.lsr_zeropagex(bus, info),
            (Instruction::LSR, AddressingMode::Absolute) => self.lsr_absolute(bus, info),
            (Instruction::LSR, AddressingMode::AbsoluteX) => self.lsr_absolutex(bus, info),
            
            // NOP
            (Instruction::NOP, _) => self.nop_implicit(info),
            
            // ORA
            (Instruction::ORA, AddressingMode::Immediate) => self.ora_immediate(bus, info),
            (Instruction::ORA, AddressingMode::ZeroPage) => self.ora_zeropage(bus, info),
            (Instruction::ORA, AddressingMode::ZeroPageX) => self.ora_zeropagex(bus, info),
            (Instruction::ORA, AddressingMode::Absolute) => self.ora_absolute(bus, info),
            (Instruction::ORA, AddressingMode::AbsoluteX) => self.ora_absolutex(bus, info),
            (Instruction::ORA, AddressingMode::AbsoluteY) => self.ora_absolutey(bus, info),
            (Instruction::ORA, AddressingMode::IndirectX) => self.ora_indirectx(bus, info),
            (Instruction::ORA, AddressingMode::IndirectY) => self.ora_indirecty(bus, info),
            
            // PHA
            (Instruction::PHA, AddressingMode::Implicit) => self.pha_implicit(bus, info),
            
            // PHP
            (Instruction::PHP, AddressingMode::Implicit) => self.php_implicit(bus, info),
            
            // PLA
            (Instruction::PLA, AddressingMode::Implicit) => self.pla_implicit(bus, info),
            
            // PLP
            (Instruction::PLP, AddressingMode::Implicit) => self.plp_implicit(bus, info),
            
            // ROL
            (Instruction::ROL, AddressingMode::Accumulator) => self.rol_accumulator(info),
            (Instruction::ROL, AddressingMode::ZeroPage) => self.rol_zeropage(bus, info),
            (Instruction::ROL, AddressingMode::ZeroPageX) => self.rol_zeropagex(bus, info),
            (Instruction::ROL, AddressingMode::Absolute) => self.rol_absolute(bus, info),
            (Instruction::ROL, AddressingMode::AbsoluteX) => self.rol_absolutex(bus, info),
            
            // ROR
            (Instruction::ROR, AddressingMode::Accumulator) => self.ror_accumulator(info),
            (Instruction::ROR, AddressingMode::ZeroPage) => self.ror_zeropage(bus, info),
            (Instruction::ROR, AddressingMode::ZeroPageX) => self.ror_zeropagex(bus, info),
            (Instruction::ROR, AddressingMode::Absolute) => self.ror_absolute(bus, info),
            (Instruction::ROR, AddressingMode::AbsoluteX) => self.ror_absolutex(bus, info),
            
            // RTI
            (Instruction::RTI, AddressingMode::Implicit) => self.rti_implicit(bus, info),
            
            // RTS
            (Instruction::RTS, AddressingMode::Implicit) => self.rts_implicit(bus, info),
            
            // SBC
            (Instruction::SBC, AddressingMode::Immediate) => self.sbc_immediate(bus, info),
            (Instruction::SBC, AddressingMode::ZeroPage) => self.sbc_zeropage(bus, info),
            (Instruction::SBC, AddressingMode::ZeroPageX) => self.sbc_zeropagex(bus, info),
            (Instruction::SBC, AddressingMode::Absolute) => self.sbc_absolute(bus, info),
            (Instruction::SBC, AddressingMode::AbsoluteX) => self.sbc_absolutex(bus, info),
            (Instruction::SBC, AddressingMode::AbsoluteY) => self.sbc_absolutey(bus, info),
            (Instruction::SBC, AddressingMode::IndirectX) => self.sbc_indirectx(bus, info),
            (Instruction::SBC, AddressingMode::IndirectY) => self.sbc_indirecty(bus, info),
            
            // SEC
            (Instruction::SEC, AddressingMode::Implicit) => self.sec_implicit(info),
            
            // SED
            (Instruction::SED, AddressingMode::Implicit) => self.sed_implicit(info),
            
            // SEI
            (Instruction::SEI, AddressingMode::Implicit) => self.sei_implicit(info),
            
            // STA
            (Instruction::STA, AddressingMode::ZeroPage) => self.sta_zeropage(bus, info),
            (Instruction::STA, AddressingMode::ZeroPageX) => self.sta_zeropagex(bus, info),
            (Instruction::STA, AddressingMode::Absolute) => self.sta_absolute(bus, info),
            (Instruction::STA, AddressingMode::AbsoluteX) => self.sta_absolutex(bus, info),
            (Instruction::STA, AddressingMode::AbsoluteY) => self.sta_absolutey(bus, info),
            (Instruction::STA, AddressingMode::IndirectX) => self.sta_indirectx(bus, info),
            (Instruction::STA, AddressingMode::IndirectY) => self.sta_indirecty(bus, info),
            
            // STX
            (Instruction::STX, AddressingMode::ZeroPage) => self.stx_zeropage(bus, info),
            (Instruction::STX, AddressingMode::ZeroPageY) => self.stx_zeropagey(bus, info),
            (Instruction::STX, AddressingMode::Absolute) => self.stx_absolute(bus, info),
            
            // STY
            (Instruction::STY, AddressingMode::ZeroPage) => self.sty_zeropage(bus, info),
            (Instruction::STY, AddressingMode::ZeroPageX) => self.sty_zeropagex(bus, info),
            (Instruction::STY, AddressingMode::Absolute) => self.sty_absolute(bus, info),
            
            // TAX
            (Instruction::TAX, AddressingMode::Implicit) => self.tax_implicit(info),
            
            // TAY
            (Instruction::TAY, AddressingMode::Implicit) => self.tay_implicit(info),
            
            // TSX
            (Instruction::TSX, AddressingMode::Implicit) => self.tsx_implicit(info),
            
            // TXA
            (Instruction::TXA, AddressingMode::Implicit) => self.txa_implicit(info),
            
            // TXS
            (Instruction::TXS, AddressingMode::Implicit) => self.txs_implicit(info),
            
            // TYA
            (Instruction::TYA, AddressingMode::Implicit) => self.tya_implicit(info),
        
            // All illegal Opcodes
            _ => self.nop_implicit(info),
        }
    }

    // =========================================================================
    //  Implement Opcode actions
    // =========================================================================
    
    // ADC
    fn adc_immediate(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let value = bus.read(self.pc);
        self.pc += 1;

        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };

        let result_u16 = (self.a as u16) + (value as u16) + carry;

        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;

        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64;
    }

    fn adc_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let value = bus.read(address as u16);
        let carry = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };
        let result = (self.a as u16) + (value as u16) + (carry as u16);

        let carry_out = result > 0xFF;
        let result = result as u8;

        let overflow = ((!(self.a ^ value) & (self.a ^ result)) & 0x80) != 0;
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(result);

        self.a = result;
        self.cycle += info.cycles as u64;
    }

    fn adc_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.x) as u16;

        let value = bus.read(final_address);

        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };

        let result_u16 = (self.a as u16) + (value as u16) + carry;

        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;

        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64;
    }

    fn adc_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);

        let value = bus.read(final_address);

        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };

        let result_u16 = (self.a as u16) + (value as u16) + carry;

        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;

        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64;
    }

    fn adc_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        let value = bus.read(final_address);

        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };

        let result_u16 = (self.a as u16) + (value as u16) + carry;

        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;

        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn adc_absolutey(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        let value = bus.read(final_address);

        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };

        let result_u16 = (self.a as u16) + (value as u16) + carry;

        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;

        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn adc_indirectx(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let ptr_with_offset = ptr.wrapping_add(self.x);

        let low = bus.read(ptr_with_offset as u16);

        let high = bus.read(ptr_with_offset.wrapping_add(1) as u16);

        let address = (high as u16) << 8 | (low as u16);

        let value = bus.read(address);

        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };

        let result_u16 = (self.a as u16) + (value as u16) + carry;

        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;

        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64;
    }

    fn adc_indirecty(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let low = bus.read(ptr as u16);
        let high = bus.read(ptr.wrapping_add(1) as u16);

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        let value = bus.read(final_address);

        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };

        let result_u16 = (self.a as u16) + (value as u16) + carry;

        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;

        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }
    
    // AND
    fn and_immediate(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let value = bus.read(self.pc);
        self.pc += 1;

        self.a &= value;
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn and_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        self.a &= bus.read(address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn and_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.x) as u16;
        self.a &= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn and_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);
        self.a &= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn and_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);
        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);
        self.a &= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn and_absolutey(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);
        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);
        self.a &= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn and_indirectx(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let ptr_with_offset = ptr.wrapping_add(self.x);
        let low = bus.read(ptr_with_offset as u16);
        let high = bus.read(ptr_with_offset.wrapping_add(1) as u16);
        let final_address = (high as u16) << 8 | (low as u16);

        self.a &= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn and_indirecty(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let low = bus.read(ptr as u16);
        let high = bus.read(ptr.wrapping_add(1) as u16);

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);
        self.a &= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }
    
    // ASL
    fn asl_accumulator(&mut self, info: &OpcodeInfo) {
        let carry_out = (self.a & 0x80) != 0;
        self.a <<= 1;
        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn asl_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let mut value = bus.read(address);
        let carry_out = (value & 0x80) != 0;
        value <<= 1;
        bus.write(address, value);

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn asl_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.x) as u16;
        let mut value = bus.read(final_address);
        let carry_out = (value & 0x80) != 0;
        value <<= 1;
        bus.write(final_address, value);

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn asl_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);
        let mut value = bus.read(final_address);
        let carry_out = (value & 0x80) != 0;
        value <<= 1;
        bus.write(final_address, value);

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn asl_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);

        let mut value = bus.read(final_address);
        let carry_out = (value & 0x80) != 0;
        value <<= 1;
        bus.write(final_address, value);

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }
    
    // Branches
    fn bcc_relative(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let offset = bus.read(self.pc) as i8;
        self.pc += 1;

        if !self.is_status_register_flag_active(StatusRegister::C) {
            let old_pc = self.pc;
            self.pc = self.pc.wrapping_add(offset as u16);
            let page_crossed = (old_pc & 0xFF00) != (self.pc & 0xFF00);

            self.cycle += info.cycles as u64 + 1 + if page_crossed { 1 } else { 0 };
        } else {
            self.cycle += info.cycles as u64;
        }
    }

    fn bcs_relative(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let offset = bus.read(self.pc) as i8;
        self.pc += 1;

        if self.is_status_register_flag_active(StatusRegister::C) {
            let old_pc = self.pc;
            self.pc = self.pc.wrapping_add(offset as u16);
            let page_crossed = (old_pc & 0xFF00) != (self.pc & 0xFF00);

            self.cycle += info.cycles as u64 + 1 + if page_crossed { 1 } else { 0 };
        } else {
            self.cycle += info.cycles as u64;
        }
    }

    fn beq_relative(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let offset = bus.read(self.pc) as i8;
        self.pc += 1;

        if self.is_status_register_flag_active(StatusRegister::Z) {
            let old_pc = self.pc;
            self.pc = self.pc.wrapping_add(offset as u16);
            let page_crossed = (old_pc & 0xFF00) != (self.pc & 0xFF00);

            self.cycle += info.cycles as u64 + 1 + if page_crossed { 1 } else { 0 };
        } else {
            self.cycle += info.cycles as u64;
        }
    }

    fn bmi_relative(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let offset = bus.read(self.pc) as i8;
        self.pc += 1;

        if self.is_status_register_flag_active(StatusRegister::N) {
            let old_pc = self.pc;
            self.pc = self.pc.wrapping_add(offset as u16);
            let page_crossed = (old_pc & 0xFF00) != (self.pc & 0xFF00);

            self.cycle += info.cycles as u64 + 1 + if page_crossed { 1 } else { 0 };
        } else {
            self.cycle += info.cycles as u64;
        }
    }

    fn bne_relative(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let offset = bus.read(self.pc) as i8;
        self.pc += 1;

        if !self.is_status_register_flag_active(StatusRegister::Z) {
            let old_pc = self.pc;
            self.pc = self.pc.wrapping_add(offset as u16);
            let page_crossed = (old_pc & 0xFF00) != (self.pc & 0xFF00);

            self.cycle += info.cycles as u64 + 1 + if page_crossed { 1 } else { 0 };
        } else {
            self.cycle += info.cycles as u64;
        }
    }

    fn bpl_relative(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let offset = bus.read(self.pc) as i8;
        self.pc += 1;

        if !self.is_status_register_flag_active(StatusRegister::N) {
            let old_pc = self.pc;
            self.pc = self.pc.wrapping_add(offset as u16);
            let page_crossed = (old_pc & 0xFF00) != (self.pc & 0xFF00);

            self.cycle += info.cycles as u64 + 1 + if page_crossed { 1 } else { 0 };
        } else {
            self.cycle += info.cycles as u64;
        }
    }

    fn bvc_relative(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let offset = bus.read(self.pc) as i8;
        self.pc += 1;

        if !self.is_status_register_flag_active(StatusRegister::V) {
            let old_pc = self.pc;
            self.pc = self.pc.wrapping_add(offset as u16);
            let page_crossed = (old_pc & 0xFF00) != (self.pc & 0xFF00);

            self.cycle += info.cycles as u64 + 1 + if page_crossed { 1 } else { 0 };
        } else {
            self.cycle += info.cycles as u64;
        }
    }

    fn bvs_relative(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let offset = bus.read(self.pc) as i8;
        self.pc += 1;

        if self.is_status_register_flag_active(StatusRegister::V) {
            let old_pc = self.pc;
            self.pc = self.pc.wrapping_add(offset as u16);
            let page_crossed = (old_pc & 0xFF00) != (self.pc & 0xFF00);

            self.cycle += info.cycles as u64 + 1 + if page_crossed { 1 } else { 0 };
        } else {
            self.cycle += info.cycles as u64;
        }
    }
    
    // BIT
    fn bit_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let value = bus.read(address);
        self.set_status_register_flag(StatusRegister::Z, (self.a & value) == 0);
        self.set_status_register_flag(StatusRegister::V, (value & 0x40) != 0);
        self.set_status_register_flag(StatusRegister::N, (value & 0x80) != 0);

        self.cycle += info.cycles as u64;
    }

    fn bit_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);
        let value = bus.read(final_address);
        self.set_status_register_flag(StatusRegister::Z, (self.a & value) == 0);
        self.set_status_register_flag(StatusRegister::V, (value & 0x40) != 0);
        self.set_status_register_flag(StatusRegister::N, (value & 0x80) != 0);
        self.cycle += info.cycles as u64;
    }
    
    // BRK
    fn brk_implicit(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        self.pc += 1;
        let high = (self.pc >> 8) as u8;
        self.push(bus, high);

        let low = (self.pc & 0xFF) as u8;
        self.push(bus, low);

        self.set_status_register_flag(StatusRegister::B, true);
        self.set_status_register_flag(StatusRegister::U, true);
        let sr_with_b = self.sr | 0x30;
        self.push(bus, sr_with_b);

        self.set_status_register_flag(StatusRegister::I, true);
        let irq_low = bus.read(0xFFFE);
        let irq_high = bus.read(0xFFFF);
        self.pc = (irq_high as u16) << 8 | (irq_low as u16);

        self.cycle += info.cycles as u64;
    }
    
    // CLx
    fn clc_implicit(&mut self, info: &OpcodeInfo) {
        self.set_status_register_flag(StatusRegister::C, false);
        self.cycle += info.cycles as u64;
    }

    fn cld_implicit(&mut self, info: &OpcodeInfo) {
        self.set_status_register_flag(StatusRegister::D, false);
        self.cycle += info.cycles as u64;
    }

    fn cli_implicit(&mut self, info: &OpcodeInfo) {
        self.set_status_register_flag(StatusRegister::I, false);
        self.cycle += info.cycles as u64;
    }

    fn clv_implicit(&mut self, info: &OpcodeInfo) {
        self.set_status_register_flag(StatusRegister::V, false);
        self.cycle += info.cycles as u64;
    }
    
    // CMP
    fn cmp_immediate(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let value = bus.read(self.pc);
        self.pc += 1;

        let result = self.a.wrapping_sub(value);

        self.set_status_register_flag(StatusRegister::C, self.a >= value);
        self.set_status_register_flag(StatusRegister::Z, self.a == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b10000000 != 0);

        self.cycle += info.cycles as u64;
    }

    fn cmp_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let value = bus.read(address as u16);
        let result = self.a.wrapping_sub(value);

        self.set_status_register_flag(StatusRegister::C, self.a >= value);
        self.set_status_register_flag(StatusRegister::Z, self.a == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b10000000 != 0);

        self.cycle += info.cycles as u64;
    }

    fn cmp_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.x);

        let value = bus.read(final_address as u16);
        let result = self.a.wrapping_sub(value);

        self.set_status_register_flag(StatusRegister::C, self.a >= value);
        self.set_status_register_flag(StatusRegister::Z, self.a == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b10000000 != 0);

        self.cycle += info.cycles as u64;
    }

    fn cmp_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);

        let value = bus.read(final_address);
        let result = self.a.wrapping_sub(value);

        self.set_status_register_flag(StatusRegister::C, self.a >= value);
        self.set_status_register_flag(StatusRegister::Z, self.a == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b10000000 != 0);

        self.cycle += info.cycles as u64;
    }

    fn cmp_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        let value = bus.read(final_address);
        let result = self.a.wrapping_sub(value);

        self.set_status_register_flag(StatusRegister::C, self.a >= value);
        self.set_status_register_flag(StatusRegister::Z, self.a == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b10000000 != 0);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn cmp_absolutey(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        let value = bus.read(final_address);
        let result = self.a.wrapping_sub(value);

        self.set_status_register_flag(StatusRegister::C, self.a >= value);
        self.set_status_register_flag(StatusRegister::Z, self.a == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b10000000 != 0);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn cmp_indirectx(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let ptr_with_offset = ptr.wrapping_add(self.x);

        let low = bus.read(ptr_with_offset as u16);
        let high = bus.read(ptr_with_offset.wrapping_add(1) as u16);

        let final_address = (high as u16) << 8 | (low as u16);

        let value = bus.read(final_address);
        let result = self.a.wrapping_sub(value);

        self.set_status_register_flag(StatusRegister::C, self.a >= value);
        self.set_status_register_flag(StatusRegister::Z, self.a == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b1000_0000 != 0);

        self.cycle += info.cycles as u64;
    }

    fn cmp_indirecty(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let low = bus.read(ptr as u16);
        let high = bus.read(ptr.wrapping_add(1) as u16);

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        let value = bus.read(final_address);
        let result = self.a.wrapping_sub(value);

        self.set_status_register_flag(StatusRegister::C, self.a >= value);
        self.set_status_register_flag(StatusRegister::Z, self.a == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b1000_0000 != 0);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }
    
    // CPX
    fn cpx_immediate(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let value = bus.read(self.pc);
        self.pc += 1;

        let result = self.x.wrapping_sub(value);
        self.set_status_register_flag(StatusRegister::C, self.x >= value);
        self.set_status_register_flag(StatusRegister::Z, self.x == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b10000000 != 0);

        self.cycle += info.cycles as u64;
    }

    fn cpx_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let value = bus.read(address);
        let result = self.x.wrapping_sub(value);

        self.set_status_register_flag(StatusRegister::C, self.x >= value);
        self.set_status_register_flag(StatusRegister::Z, self.x == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b10000000 != 0);

        self.cycle += info.cycles as u64;
    }

    fn cpx_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);
        let value = bus.read(final_address);
        let result = self.x.wrapping_sub(value);

        self.set_status_register_flag(StatusRegister::C, self.x >= value);
        self.set_status_register_flag(StatusRegister::Z, self.x == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b10000000 != 0);

        self.cycle += info.cycles as u64;
    }
    
    // CPY
    fn cpy_immediate(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let value = bus.read(self.pc);
        self.pc += 1;

        let result = self.y.wrapping_sub(value);
        self.set_status_register_flag(StatusRegister::C, self.y >= value);
        self.set_status_register_flag(StatusRegister::Z, self.y == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b10000000 != 0);

        self.cycle += info.cycles as u64;
    }

    fn cpy_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let value = bus.read(address);
        let result = self.y.wrapping_sub(value);

        self.set_status_register_flag(StatusRegister::C, self.y >= value);
        self.set_status_register_flag(StatusRegister::Z, self.y == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b10000000 != 0);

        self.cycle += info.cycles as u64;
    }

    fn cpy_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);
        let value = bus.read(final_address);
        let result = self.y.wrapping_sub(value);

        self.set_status_register_flag(StatusRegister::C, self.y >= value);
        self.set_status_register_flag(StatusRegister::Z, self.y == value);
        self.set_status_register_flag(StatusRegister::N, result & 0b10000000 != 0);

        self.cycle += info.cycles as u64;
    }
    
    // DEC
    fn dec_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let value = bus.read(address).wrapping_sub(1);
        bus.write(address, value);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn dec_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.x) as u16;
        let value = bus.read(final_address).wrapping_sub(1);
        bus.write(final_address, value);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn dec_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);
        let value = bus.read(final_address).wrapping_sub(1);
        bus.write(final_address, value);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn dec_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);
        let value = bus.read(final_address).wrapping_sub(1);
        bus.write(final_address, value);

        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64;
    }
    
    // DEX, DEY
    fn dex_implicit(&mut self, info: &OpcodeInfo) {
        self.x = self.x.wrapping_sub(1);
        self.update_status_register_nz(self.x);
        self.cycle += info.cycles as u64;
    }

    fn dey_implicit(&mut self, info: &OpcodeInfo) {
        self.y = self.y.wrapping_sub(1);
        self.update_status_register_nz(self.y);
        self.cycle += info.cycles as u64;
    }
    
    // EOR
    fn eor_immediate(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let value = bus.read(self.pc);
        self.pc += 1;

        self.a ^= value;
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn eor_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        self.a ^= bus.read(address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn eor_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.x) as u16;
        self.a ^= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn eor_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);
        self.a ^= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn eor_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);
        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);
        self.a ^= bus.read(final_address);

        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn eor_absolutey(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);
        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        self.a ^= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn eor_indirectx(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let ptr_with_offset = ptr.wrapping_add(self.x);
        let low = bus.read(ptr_with_offset as u16);
        let high = bus.read(ptr_with_offset.wrapping_add(1) as u16);
        let final_address = (high as u16) << 8 | (low as u16);

        self.a ^= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn eor_indirecty(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let low = bus.read(ptr as u16);
        let high = bus.read(ptr.wrapping_add(1) as u16);
        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        self.a ^= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }
    
    // INC
    fn inc_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) { 
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let value = bus.read(address);
        let value = value.wrapping_add(1);
        bus.write(address, value);

        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64;
     }

    fn inc_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) { 
        let address = bus.read(self.pc);
        self.pc += 1;

        let address_with_offset = address.wrapping_add(self.x) as u16;

        let value = bus.read(address_with_offset);
        let value = value.wrapping_add(1);
        bus.write(address_with_offset, value);

        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64;
    }

    fn inc_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) { 
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let address = ((high as u16) << 8) | (low as u16);
        let value = bus.read(address);
        let value = value.wrapping_add(1);
        bus.write(address, value);

        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64;
    }

    fn inc_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) { 
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = ((high as u16) << 8) | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);

        let value = bus.read(final_address);
        let new_value = value.wrapping_add(1);
        bus.write(final_address, new_value);

        self.update_status_register_nz(new_value);
        self.cycle += info.cycles as u64;
    }

    // INX, INY
    fn inx_implicit(&mut self, info: &OpcodeInfo) { 
        self.x = self.x.wrapping_add(1);
        self.update_status_register_nz(self.x);
        self.cycle += info.cycles as u64;
    }

    fn iny_implicit(&mut self, info: &OpcodeInfo) { 
        self.y = self.y.wrapping_add(1);
        self.update_status_register_nz(self.y);
        self.cycle += info.cycles as u64;
    }
    
    // JMP
    fn jmp_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low_addess = bus.read(self.pc);
        self.pc +=1;

        let high_address = bus.read(self.pc);
        self.pc += 1;

        let jump_address = (high_address as u16) << 8 | (low_addess as u16);
        self.pc = jump_address;

        self.cycle += info.cycles as u64;
    }

    fn jmp_indirect(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low_ptr = bus.read(self.pc);
        self.pc += 1;

        let high_ptr = bus.read(self.pc);
        self.pc += 1;

        let ptr_address = (high_ptr as u16)  << 8 | (low_ptr as u16);

        let low_addess = bus.read(ptr_address);

        let high_byte_addr = (ptr_address & 0xFF00) | ((ptr_address + 1) & 0x00FF);
        let high_address = bus.read(high_byte_addr);
        let jump_address = (high_address as u16) << 8 | (low_addess as u16);
        self.pc = jump_address;

        self.cycle += info.cycles as u64;
    }
    
    // JSR
    fn jsr_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low_address = bus.read(self.pc);
        self.pc += 1;

        let high_address = bus.read(self.pc);
        self.pc += 1;

        let jsr_address = (high_address as u16) << 8 | (low_address as u16);

        let high: u8 = ((self.pc - 1) >> 8) as u8;
        self.push(bus, high);

        let low: u8 = (self.pc - 1) as u8;
        self.push(bus, low);

        self.pc = jsr_address;

        self.cycle += info.cycles as u64;
    }
    
    // LDA
    fn lda_immediate(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let value = bus.read(self.pc);
        self.pc += 1;
        
        self.a = value;
        
        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64;
    }

    fn lda_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let value = bus.read(address);
        self.a = value;

        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64;
    }

    fn lda_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let address_with_offset = address.wrapping_add(self.x) as u16;

        let value = bus.read(address_with_offset);
        self.a = value;

        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64;
    }

    fn lda_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let address = (high as u16) << 8 | (low as u16);

        let value = bus.read(address);
        self.a = value;

        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64;
    }

    fn lda_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        let value = bus.read(final_address);
        self.a = value;

        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn lda_absolutey(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        let value = bus.read(final_address);
        self.a = value;

        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn lda_indirectx(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let ptr_final = ptr.wrapping_add(self.x);

        let addr_low = bus.read(ptr_final as u16);
        let addr_high = bus.read(ptr_final.wrapping_add(1) as u16);

        let address = (addr_high as u16) << 8 | (addr_low as u16);

        let value = bus.read(address);
        self.a = value;

        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64;
    }

    fn lda_indirecty(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let addr_low = bus.read(ptr as u16);
        let addr_high = bus.read(ptr.wrapping_add(1) as u16);

        let base_address = (addr_high as u16) << 8 | (addr_low as u16);

        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        let value = bus.read(final_address);
        self.a = value;

        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }
    
    // LDX
    fn ldx_immediate(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let value = bus.read(self.pc);
        self.pc += 1;

        self.x = value;
        self.update_status_register_nz(self.x);

        self.cycle += info.cycles as u64;
    }

    fn ldx_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        self.x = bus.read(address);
        self.update_status_register_nz(self.x);

        self.cycle += info.cycles as u64;
    }

    fn ldx_zeropagey(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.y) as u16;

        self.x = bus.read(final_address);
        self.update_status_register_nz(self.x);

        self.cycle += info.cycles as u64;
    }

    fn ldx_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);

        self.x = bus.read(final_address);
        self.update_status_register_nz(self.x);

        self.cycle += info.cycles as u64;
    }

    fn ldx_absolutey(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        self.x = bus.read(final_address);
        self.update_status_register_nz(self.x);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }
    
    // LDY
    fn ldy_immediate(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let value = bus.read(self.pc);
        self.pc += 1;

        self.y = value;
        self.update_status_register_nz(self.y);

        self.cycle += info.cycles as u64;
    }

    fn ldy_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        self.y = bus.read(address);
        self.update_status_register_nz(self.y);

        self.cycle += info.cycles as u64;
    }

    fn ldy_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.x) as u16;
        self.y = bus.read(final_address);
        self.update_status_register_nz(self.y);

        self.cycle += info.cycles as u64;
    }

    fn ldy_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);
        self.y = bus.read(final_address);
        self.update_status_register_nz(self.y);

        self.cycle += info.cycles as u64;
    }

    fn ldy_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        self.y = bus.read(final_address);
        self.update_status_register_nz(self.y);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }
    
    // LSR
    fn lsr_accumulator(&mut self, info: &OpcodeInfo) {
        let carry_out = (self.a & 0x01) != 0;
        self.a >>= 1;
        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn lsr_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let mut value = bus.read(address);
        let carry_out = (value & 0x01) != 0;
        value >>= 1;
        bus.write(address, value);

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn lsr_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.x) as u16;
        let mut value = bus.read(final_address);
        let carry_out = (value & 0x01) != 0;
        value >>= 1;
        bus.write(final_address, value);

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn lsr_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);
        let mut value = bus.read(final_address);
        let carry_out = (value & 0x01) != 0;
        value >>= 1;
        bus.write(final_address, value);

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn lsr_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);

        let mut value = bus.read(final_address);
        let carry_out = (value & 0x01) != 0;
        value >>= 1;
        bus.write(final_address, value);

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);
        self.cycle += info.cycles as u64;
    }
    
    // NOP
    fn nop_implicit(&mut self, info: &OpcodeInfo) {
        self.cycle += info.cycles as u64;
    }

    // ORA
    fn ora_immediate(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let value = bus.read(self.pc);
        self.pc += 1;

        self.a |= value;
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn ora_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        self.a |= bus.read(address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn ora_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.x) as u16;
        self.a |= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn ora_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);

        self.a |= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn ora_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        self.a |= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn ora_absolutey(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        self.a |= bus.read(final_address);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn ora_indirectx(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let ptr_with_offset = ptr.wrapping_add(self.x);
        let low = bus.read(ptr_with_offset as u16);
        let high = bus.read(ptr_with_offset.wrapping_add(1) as u16);

        let final_address = (high as u16) << 8 | (low as u16);
        self.a |= bus.read(final_address);

        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn ora_indirecty(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let low = bus.read(ptr as u16);
        let high = bus.read(ptr.wrapping_add(1) as u16);

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        self.a |= bus.read(final_address);

        self.update_status_register_nz(self.a);
        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }
    
    // PHA, PHP, PLA, PLP
    fn pha_implicit(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        self.push(bus, self.a);
        self.cycle += info.cycles as u64;
    }

    fn php_implicit(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let sr_to_push = self.sr | 0x30;
        self.push(bus, sr_to_push);
        self.cycle += info.cycles as u64;
    }

    fn pla_implicit(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        self.a = self.pull(bus);
        self.update_status_register_nz(self.a);
        self.cycle += info.cycles as u64;
    }


    fn plp_implicit(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        self.sr = self.pull(bus);

        // For CPU 6502 the Flag B ignored (reset to 0)
        self.set_status_register_flag(StatusRegister::B, false);
        self.set_status_register_flag(StatusRegister::U, true);

        self.cycle += info.cycles as u64;
    }

    // ROL
    fn rol_accumulator(&mut self, info: &OpcodeInfo) {
        let carry_in = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };
        let carry_out = (self.a & 0x80) != 0;
        self.a = (self.a << 1) | carry_in;

        self.set_status_register_flag(StatusRegister::C, carry_out);

        self.update_status_register_nz(self.a);
        self.cycle += info.cycles as u64;
    }

    fn rol_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let mut value = bus.read(address);
        let carry_in = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };
        let carry_out = (value & 0x80) != 0;

        value = (value << 1) | carry_in;
        bus.write(address, value);

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn rol_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.x) as u16;
        let mut value = bus.read(final_address);
        let carry_in = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };
        let carry_out = (value & 0x80) != 0;

        value = (value << 1) | carry_in;
        bus.write(final_address, value);

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn rol_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);
        let mut value = bus.read(final_address);

        let carry_in = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };
        let carry_out = (value & 0x80) != 0;

        value = (value << 1) | carry_in;
        bus.write(final_address, value);

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn rol_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);
        let mut value = bus.read(final_address);

        let carry_in = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };
        let carry_out = (value & 0x80) != 0;
        value = (value << 1) | carry_in;

        bus.write(final_address, value);
        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }
    
    // ROR
    fn ror_accumulator(&mut self, info: &OpcodeInfo) {
        let carry_in = if self.is_status_register_flag_active(StatusRegister::C) { 0x80 } else { 0 };
        let carry_out = (self.a & 0x01) != 0;

        self.a = (self.a >> 1) | carry_in;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(self.a);

        self.cycle += info.cycles as u64;
    }

    fn ror_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let mut value = bus.read(address);
        let carry_in = if self.is_status_register_flag_active(StatusRegister::C) { 0x80 } else { 0 };
        let carry_out = (value & 0x01) != 0;

        value = (value >> 1) | carry_in;

        bus.write(address, value);
        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn ror_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.x) as u16;
        let mut value = bus.read(final_address);

        let carry_in = if self.is_status_register_flag_active(StatusRegister::C) { 0x80 } else { 0 };
        let carry_out = (value & 0x01) != 0;
        value = (value >> 1) | carry_in;

        bus.write(final_address, value);
        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn ror_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);
        let mut value = bus.read(final_address);

        let carry_in = if self.is_status_register_flag_active(StatusRegister::C) { 0x80 } else { 0 };
        let carry_out = (value & 0x01) != 0;

        value = (value >> 1) | carry_in;

        bus.write(final_address, value);
        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }

    fn ror_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);
        let mut value = bus.read(final_address);

        let carry_in = if self.is_status_register_flag_active(StatusRegister::C) { 0x80 } else { 0 };
        let carry_out = (value & 0x01) != 0;

        value = (value >> 1) | carry_in;
        bus.write(final_address, value);

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.update_status_register_nz(value);

        self.cycle += info.cycles as u64;
    }
    
    // RTI, RTS
    fn rti_implicit(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        self.sr = self.pull(bus);
        
        // For CPU 6502 the Flag B ignored (reset to 0)
        self.set_status_register_flag(StatusRegister::B, false);
        self.set_status_register_flag(StatusRegister::U, true);

        let low_pc = self.pull(bus);
        let high_pc = self.pull(bus);
        self.pc = (high_pc as u16) << 8 | (low_pc as u16);

        self.cycle += info.cycles as u64;
    }
    
    fn rts_implicit(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low_pc = self.pull(bus);
        let high_pc = self.pull(bus);

        self.pc = ((high_pc as u16) << 8 | (low_pc as u16)) + 1;
        self.cycle += info.cycles as u64;
    }
    
    // SBC
    fn sbc_immediate(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let value = bus.read(self.pc);
        self.pc += 1;
        let value = value ^ 0xFF;

        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };
        let result_u16 = (self.a as u16) + (value as u16) + carry;
        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;
        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64;
    }

    fn sbc_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;

        let value = bus.read(address) ^ 0xFF;
        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };

        let result_u16 = (self.a as u16) + (value as u16) + carry;
        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;
        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64;
    }

    fn sbc_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;

        let final_address = address.wrapping_add(self.x) as u16;
        let value = bus.read(final_address) ^ 0xFF;

        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };

        let result_u16 = (self.a as u16) + (value as u16) + carry;
        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;
        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64;
    }

    fn sbc_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let final_address = (high as u16) << 8 | (low as u16);

        let value = bus.read(final_address) ^ 0xFF;
        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };
        let result_u16 = (self.a as u16) + (value as u16) + carry;

        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;
        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64;
    }

    fn sbc_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        let value = bus.read(final_address) ^ 0xFF;
        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };
        let result_u16 = (self.a as u16) + (value as u16) + carry;
        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;

        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn sbc_absolutey(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;

        let high = bus.read(self.pc);
        self.pc += 1;

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        let value = bus.read(final_address) ^ 0xFF;
        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };
        let result_u16 = (self.a as u16) + (value as u16) + carry;
        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;
        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }

    fn sbc_indirectx(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let ptr_with_offset = ptr.wrapping_add(self.x);

        let low = bus.read(ptr_with_offset as u16);
        let high = bus.read(ptr_with_offset.wrapping_add(1) as u16);
        let final_address = (high as u16) << 8 | (low as u16);

        let value = bus.read(final_address) ^ 0xFF;
        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };

        let result_u16 = (self.a as u16) + (value as u16) + carry;
        let result_u8 = result_u16 as u8;

        let carry_out = result_u16 > 0xFF;
        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64;
    }

    fn sbc_indirecty(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;

        let low = bus.read(ptr as u16);
        let high = bus.read(ptr.wrapping_add(1) as u16);

        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);

        let page_crossed = (base_address & 0xFF00) != (final_address & 0xFF00);

        let value = bus.read(final_address) ^ 0xFF;
        let carry: u16 = if self.is_status_register_flag_active(StatusRegister::C) { 1 } else { 0 };

        let result_u16 = (self.a as u16) + (value as u16) + carry;
        let result_u8 = result_u16 as u8;
        let carry_out = result_u16 > 0xFF;
        let overflow = ((!(self.a ^ value) & (self.a ^ result_u8)) & 0x80) != 0;

        self.set_status_register_flag(StatusRegister::C, carry_out);
        self.set_status_register_flag(StatusRegister::V, overflow);
        self.update_status_register_nz(result_u8);

        self.a = result_u8;
        self.cycle += info.cycles as u64 + if page_crossed { 1 } else { 0 };
    }
    
    // SEC, SED, SEI
    fn sec_implicit(&mut self, info: &OpcodeInfo) {
        self.set_status_register_flag(StatusRegister::C, true);
        self.cycle += info.cycles as u64;
    }

    fn sed_implicit(&mut self, info: &OpcodeInfo) {
        self.set_status_register_flag(StatusRegister::D, true);
        self.cycle += info.cycles as u64;
    }

    fn sei_implicit(&mut self, info: &OpcodeInfo) {
        self.set_status_register_flag(StatusRegister::I, true);
        self.cycle += info.cycles as u64;
    }
    
    // STA
    fn sta_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;
        bus.write(address, self.a);
        self.cycle += info.cycles as u64;
    }

    fn sta_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;
        let final_address = address.wrapping_add(self.x) as u16;
        bus.write(final_address, self.a);
        self.cycle += info.cycles as u64;
    }

    fn sta_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;
        let high = bus.read(self.pc);
        self.pc += 1;
        let final_address = (high as u16) << 8 | (low as u16);
        bus.write(final_address, self.a);
        self.cycle += info.cycles as u64;
    }

    fn sta_absolutex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;
        let high = bus.read(self.pc);
        self.pc += 1;
        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.x as u16);
        bus.write(final_address, self.a);
        self.cycle += info.cycles as u64;
    }

    fn sta_absolutey(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;
        let high = bus.read(self.pc);
        self.pc += 1;
        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);
        bus.write(final_address, self.a);
        self.cycle += info.cycles as u64;
    }

    fn sta_indirectx(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;
        let ptr_with_offset = ptr.wrapping_add(self.x);
        let low = bus.read(ptr_with_offset as u16);
        let high = bus.read(ptr_with_offset.wrapping_add(1) as u16);
        let final_address = (high as u16) << 8 | (low as u16);
        bus.write(final_address, self.a);
        self.cycle += info.cycles as u64;
    }

    fn sta_indirecty(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let ptr = bus.read(self.pc);
        self.pc += 1;
        let low = bus.read(ptr as u16);
        let high = bus.read(ptr.wrapping_add(1) as u16);
        let base_address = (high as u16) << 8 | (low as u16);
        let final_address = base_address.wrapping_add(self.y as u16);
        bus.write(final_address, self.a);
        self.cycle += info.cycles as u64;
    }
    
    // STX
    fn stx_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;
        bus.write(address, self.x);
        self.cycle += info.cycles as u64;
    }

    fn stx_zeropagey(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;
        let final_address = address.wrapping_add(self.y) as u16;
        bus.write(final_address, self.x);
        self.cycle += info.cycles as u64;
    }

    fn stx_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;
        let high = bus.read(self.pc);
        self.pc += 1;
        let final_address = (high as u16) << 8 | (low as u16);
        bus.write(final_address, self.x);
        self.cycle += info.cycles as u64;
    }
    
    // STY
    fn sty_zeropage(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc) as u16;
        self.pc += 1;
        bus.write(address, self.y);
        self.cycle += info.cycles as u64;
    }

    fn sty_zeropagex(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let address = bus.read(self.pc);
        self.pc += 1;
        let final_address = address.wrapping_add(self.x) as u16;
        bus.write(final_address, self.y);
        self.cycle += info.cycles as u64;
    }

    fn sty_absolute(&mut self, bus: &mut Bus, info: &OpcodeInfo) {
        let low = bus.read(self.pc);
        self.pc += 1;
        let high = bus.read(self.pc);
        self.pc += 1;
        let final_address = (high as u16) << 8 | (low as u16);
        bus.write(final_address, self.y);
        self.cycle += info.cycles as u64;
    }
    
    // Transferts
    fn tax_implicit(&mut self, info: &OpcodeInfo) {
        self.x = self.a;
        self.update_status_register_nz(self.x);
        self.cycle += info.cycles as u64;
    }

    fn tay_implicit(&mut self, info: &OpcodeInfo) {
        self.y = self.a;
        self.update_status_register_nz(self.y);
        self.cycle += info.cycles as u64;
    }

    fn tsx_implicit(&mut self, info: &OpcodeInfo) {
        self.x = self.sp;
        self.update_status_register_nz(self.x);
        self.cycle += info.cycles as u64;
    }

    fn txa_implicit(&mut self, info: &OpcodeInfo) {
        self.a = self.x;
        self.update_status_register_nz(self.a);
        self.cycle += info.cycles as u64;
    }

    fn txs_implicit(&mut self, info: &OpcodeInfo) {
        self.sp = self.x;
        self.cycle += info.cycles as u64;
    }

    fn tya_implicit(&mut self, info: &OpcodeInfo) {
        self.a = self.y;
        self.update_status_register_nz(self.a);
        self.cycle += info.cycles as u64;
    }
}

#[cfg(test)]
mod test {
   use super::*;
   use crate::{bus::Bus, cartridge::Cartridge, mapper_nrom::MapperNROM};
 
   #[test]
   fn test_0xa9_lda_immediate_load_data() {
        //instructions [0xa9, 0x05, 0x00]
        let instruction = vec![0xa9, 0x05, 0x00];

        let bus = &mut Bus::new();

        let cartridge = Cartridge {
            prg_rom: instruction.clone(),
            chr_rom: vec![],
            mapper: 0,
            mirroring_type: crate::cartridge::Mirroring::Horizontal
        };

        bus.cartridge = Some(cartridge);
        bus.mapper = Some(Box::new(MapperNROM::new()));

        let mut cpu: Cpu = Cpu::init();
        cpu.pc = 0x8000;

        let mut index = 0;
        while index < instruction.len() {
            cpu.execute(bus);
            index += 1;
        }

        assert_eq!(cpu.a, 0x05);
        assert!(cpu.sr & 0b0000_0010 == 0b00);
        assert!(cpu.sr & 0b1000_0000 == 0);
   }

   #[test]
    fn test_0xa9_lda_zero_flag() {
        let instruction = vec![0xa9, 0x00, 0x00];

        let bus = &mut Bus::new();
        let cartridge = Cartridge {
            prg_rom: instruction.clone(),
            chr_rom: vec![],
            mapper: 0,
            mirroring_type: crate::cartridge::Mirroring::Horizontal
        };

        bus.cartridge = Some(cartridge);
        bus.mapper = Some(Box::new(MapperNROM::new()));

        let mut cpu: Cpu = Cpu::init();
        cpu.pc = 0x8000;

        let mut index = 0;
        while index < instruction.len() {
            cpu.execute(bus);
            index += 1;
        }
        assert!(cpu.sr & 0b0000_0010 == 0b10);
    }

   #[test]
   fn test_0xaa_tax_move_a_to_x() {
        let instruction: Vec<u8> = vec![0xaa, 0x00];

        let bus = &mut Bus::new();
        let cartridge = Cartridge {
            prg_rom: instruction.clone(),
            chr_rom: vec![],
            mapper: 0,
            mirroring_type: crate::cartridge::Mirroring::Horizontal
        };

        bus.cartridge = Some(cartridge);
        bus.mapper = Some(Box::new(MapperNROM::new()));

        let mut cpu: Cpu = Cpu::init();
        cpu.pc = 0x8000;
        cpu.a = 10;

        let mut index = 0;
        while index < instruction.len() {
            cpu.execute(bus);
            index += 1;
        } 

        assert_eq!(cpu.x, 10);
   }

    #[test]
   fn test_5_ops_working_together() {
        let instruction: Vec<u8> = vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00];

        let bus = &mut Bus::new();
        
        let cartridge = Cartridge {
            prg_rom: instruction.clone(),
            chr_rom: vec![],
            mapper: 0,
            mirroring_type: crate::cartridge::Mirroring::Horizontal
        };

        bus.cartridge = Some(cartridge);
        bus.mapper = Some(Box::new(MapperNROM::new()));

        let mut cpu: Cpu = Cpu::init();
        cpu.pc = 0x8000;
        
        let mut index = 0;
        while index < instruction.len() {
            cpu.execute(bus);
            index += 1;
        }

        assert_eq!(cpu.x, 0xc1);
   }

    #[test]
    fn test_inx_overflow() {
        let instruction: Vec<u8> = vec![0xe8, 0xe8, 0x00];

        let bus = &mut Bus::new();

        let cartridge = Cartridge {
            prg_rom: instruction.clone(),
            chr_rom: vec![],
            mapper: 0,
            mirroring_type: crate::cartridge::Mirroring::Horizontal
        };

        bus.cartridge = Some(cartridge);
        bus.mapper = Some(Box::new(MapperNROM::new()));


        let mut cpu: Cpu = Cpu::init();
        cpu.pc = 0x8000;
        cpu.x = 0xff;

        let mut index = 0;
        while index < instruction.len() {
            cpu.execute(bus);
            index += 1;
        }

        assert_eq!(cpu.x, 1);
    }

}