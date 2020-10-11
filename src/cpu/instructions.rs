use super::register::Flags;

pub(crate) enum Instruction {
    ADC(AddressMode),
    AND(AddressMode),
    BIT(AddressMode),
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLR(Flags),
    CMP(AddressMode),
    CPX(AddressMode),
    CPY(AddressMode),
    DEC(AddressMode),
    DEX,
    DEY,
    EOR(AddressMode),
    INC(AddressMode),
    INX,
    INY,
    JMP(AddressMode),
    JSR,
    LDA(AddressMode),
    LDX(AddressMode),
    LDY(AddressMode),
    LSR,
    NOP,
    ORA(AddressMode),
    PHA,
    PHP,
    PLA,
    PLP,
    ROL(AddressMode),
    ROR(AddressMode),
    RIT,
    RTS,
    SBC(AddressMode),
    SET(Flags),
    STA(AddressMode),
    STX(AddressMode),
    STY(AddressMode),
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

pub(crate) enum AddressMode {
    Accumulator,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Immediate,
    Indirect,
    XIndirect,
    IndirectY,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}

pub(crate) fn decode(opcode: u8) -> Instruction {
    match opcode {
        _ => panic!("Not implemented"),
    }
}


