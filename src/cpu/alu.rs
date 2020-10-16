/// Handles operation of the arithmetic operations within the CPU
/// and keeps behaviour of flags in one place.
/// 
/// This isn't modelled exactly on the phyiscal operation of the 6502.
/// this is only a way to keep the emulator code modular

/// Returns the result of adding a + b with carry_in, giving a result
/// (value, carry_out, overflow)
pub fn adc(a: u8, b: u8, carry_in: bool) -> (u8, bool, bool) {
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

pub fn and(a: u8, b: u8) -> u8 {
    a & b
}

pub fn asl(a: u8) -> (u8, bool) {
    let (value, carry, _) = adc(a, a, false);
    (value, carry)
}

// This is basically sbc without overflow or carry in
pub fn cmp(a: u8, b: u8) -> (u8, bool) {
    let (value, carry, _) = adc(a, !b, false);
    (value, carry)
}

pub fn dec(a: u8) -> u8 {
    let (value, _, _) = adc(a, 0xff, false);
    value
}

pub fn eor(a: u8, b: u8) -> u8 {
    a ^ b
}

pub fn lsr(a: u8) -> (u8, bool) {
    a.overflowing_shr(1)
}

pub fn inc(a: u8) -> u8 {
    let (value, _, _) = adc(a, 1, false);
    value
}

pub fn or(a: u8, b: u8) -> u8 {
    a | b
}

pub fn rol(a: u8, carry_in: bool) -> (u8, bool) {
    let (value, carry, _) = adc(a, a, carry_in);
    (value, carry)
}

/// Returns the result of shifting `a' right by one bit,
/// Making bit 7 of a equal to carry_in,
/// returning (shifted_a, carry_out)
/// where carry_out is the old bit 0 of `a'
pub fn ror(a: u8, carry_in: bool) -> (u8, bool) {
    let (value, carry_out) = a.overflowing_shr(1);
    let carry = if carry_in {0b10000000} else {0};
    (value | carry, carry_out)
}

pub fn sbc(a: u8, b: u8, carry_in: bool) -> (u8, bool, bool) {
    adc(a, !b, carry_in)
}