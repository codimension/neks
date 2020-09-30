pub struct Register16(u16);
pub struct Register8(u8);

#[allow(non_snake_case)]
pub struct CPU {
    A: Register8,
    X: Register8,
    Y: Register8
    PC: Register16,
}

impl CPU {
    pub fn init() -> () {
        let cpu = Self{A: 0, X:0, Y:0, PC: 0};
        loop {}
    }
}
