use std::convert::From;
use std::ops::{Add,AddAssign};

#[derive(Debug,Clone,Copy)]
pub struct Register16(u16);
#[derive(Debug,Clone,Copy)]
pub struct Register8(u8);


impl From<Register16> for u16 {
    fn from(reg: Register16) -> Self {
        reg.0
    }
}

impl Register8 {
    pub fn new(val: u8) -> Self {
        Self(val)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

impl Register16 {
    pub fn new(val: u16) -> Self {
        Self(val)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }
}