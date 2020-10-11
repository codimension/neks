use bitflags::*;
use std::convert::From;

// It's customary when dealing with hardware registers to use upper case
#[allow(non_snake_case)]
/// Register bank within the CPU
pub(crate) struct Registers {
    /// Accumulator register
    pub A: u8,
    /// X index register
    pub X: u8,
    /// Y index register
    pub Y: u8,
    /// Stack pointer
    pub S: u8,
    /// Processor flags register
    pub P: Flags,
}

impl Registers {
    pub fn init() -> Self {
        Self {
            A: 0,
            X: 0,
            Y: 0,
            S: 0,
            P: Flags::from(0),
        }
    }
}

bitflags! {
    /// Represents the bit layout of the Flags register
    pub(crate) struct Flags: u8 {
        const N = 0b00000001;
        const V = 0b00000010;
        const B = 0b00001000;
        const D = 0b00010000;
        const I = 0b00100000;
        const Z = 0b01000000;
        const C = 0b10000000;
    }
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        Self::from_bits_truncate(value)
    }
}

impl From<Flags> for u8 {
    fn from(flags: Flags) -> Self {
        flags.bits
    }
}