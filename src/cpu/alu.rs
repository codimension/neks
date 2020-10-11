/// Handles operation of the arithmetic operations within the CPU
/// and keeps behaviour of flags in one place.
/// 
/// This isn't modelled exactly on the phyiscal operation of the 6502.
/// That is, I don't know which of these are actually implemented in the APU of the 6502,
/// this is only a way to keep the emulator code modular
use super::register::Flags;

pub(crate) enum Op {
    Adc,
    Asl,
    And,
    Cmp,
    Dec,
    Eor,
    Inc,
    Or,
    Rol,
    Ror,
    Sbc,
}

/// Returns the result of adding a + b with carry_in, giving a result
/// (value, carry_out, overflow)
fn adc(a: u8, b: u8, carry_in: bool) -> (u8, bool, bool) {
    let (value, carry_out) = match carry_in {
        true => {
            let (v, c1) = a.overflowing_add(b);
            let (v, c2) = v.overflowing_add(1);
            (v, c1 || c2)
        },
        false => a.overflowing_add(b)
    };
    let overflow = (a ^ value) & (b ^ value) & 0x80 == 0;
    (value, carry_out, overflow)
}

/// Returns the result of shifting `a' right by one bit,
/// Making bit 7 of a equal to carry_in,
/// returning (shifted_a, carry_out)
/// where carry_out is the old bit 0 of `a'
fn ror(a: u8, carry_in: bool) -> (u8, bool) {
    let (value, carry_out) = a.overflowing_shr(1);
    let carry = if carry_in {0b10000000} else {0};
    (value | carry, carry_out)
}

/// Represents the Arithmetic Logic Unit of the 6502
pub(crate) struct ALU {
    /// One of the two input registers to the 6502 ALU
    pub input_a: u8,
    /// One of the two input registers to the 6502 ALU
    pub input_b: u8,
    /// The carry bit within the 6502 ALU, not necessarily taken from the processor status register
    pub carry_in: bool,
    pub flags: Flags,
    /// The output byte, which will be written back to the accumulator or memory by the CPU
    pub output: u8,
    pub carry_out: bool,
    pub overflow_out: bool,
}

impl ALU {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            input_a: 0,
            input_b: 0,
            carry_in: false,
            flags: Flags::from(0),
            output: 0,
            carry_out: false,
            overflow_out: false,
        }
    }

    #[inline]
    fn adc(&mut self, a: u8, b: u8) {
        let (value, carry, overflow) = adc(a, b, self.carry_in);
        self.carry_out = carry;
        self.overflow_out = overflow;
        self.output = value;
    }

    #[inline]
    pub(crate) fn do_arithmetic(&mut self, op: Op) -> (u8, Flags) {
        // Assumes that the CPU has set input_a and input_b (if necessary)
        // Relies on the CPU to use the output
        // Copies the carry and overflow flags to 'flags' only if requested by the call from the CPU
        // The pattern match replaces the decoding logic of the 6502

        // Hold the flags to be passed back to the processor status register
        // The zero and negative flags are always passed back so operations here only need to set
        // the carry and overflow flags if these should be return back to CPU.
        let mut flags = Flags::from(0);

        match op {
            Op::Adc => {
                self.carry_in = self.flags.contains(Flags::C);
                self.adc(self.input_a, self.input_b);
                self.flags.set(Flags::C, self.carry_out);
                self.flags.set(Flags::V, self.overflow_out);
            },
            Op::And => {
                self.output = self.input_a & self.input_b;
            },
            Op::Asl => {
                // Shift left by adding to self
                // Takes care of carry nicely
                self.carry_in = false;
                self.adc(self.input_a, self.input_a);
                flags = Flags::C;
            },
            Op::Cmp => {
                // See SBC. 
                // Same as this except we
                // - Don't use the borrow
                // - Don't return the overflow result to the CPU
                self.carry_in = false;
                self.adc(self.input_a, !self.input_b);
                self.flags.set(Flags::C, self.carry_out);
            },
            Op::Dec => {
                self.adc(self.input_a, 0xff);
            },
            Op::Eor => {
                self.output = self.input_a ^ self.input_b;
            },
            Op::Inc => {
                self.adc(self.input_a, 1);
            },
            Op::Or => {
                self.output = self.input_a | self.input_b;
            },
            Op::Rol => {
                // This will use the existing carry, so works correctly
                self.adc(self.input_a, self.input_a);
            },
            Op::Ror => {
                let (value, carry) = ror(self.input_a, self.carry_in);
                self.output = value;
                self.carry_out = carry;
            },
            Op::Sbc => {
                // Subtraction can be implemented simply by complementing b and doing addition
                self.carry_in = self.flags.contains(Flags::C);
                self.adc(self.input_a, !self.input_b);
                self.flags.set(Flags::C, self.carry_out);
                self.flags.set(Flags::V, self.overflow_out);
            },
            _ => panic!("Unimplemented ALU instruction"),
        }

        if flags.contains(Flags::C) {
            self.flags.set(Flags::C, self.carry_out);
        }
        if flags.contains(Flags::V) {
            self.flags.set(Flags::V, self.overflow_out);
        }
        if self.output == 0 {
            self.flags.insert(Flags::Z);
        }
        if self.output & 0b10000000 == 0b10000000 {
            self.flags.insert(Flags::N);
        }

        (self.output, self.flags)
    }
}