pub enum AddressingMode {
    Accumulator,
    AbsoluteX,
    AbsoluteY,
    Absolute,
    Immediate,
    Implied,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}

pub fn decode(byte: u8) -> AddressingMode {
    let a = byte & 0b11100000;
    let b = byte & 0b00011100;
    let c = byte & 0b00000011;

    match b {
        
    }
}
