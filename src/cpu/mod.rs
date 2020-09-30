pub struct Register16(u16);
pub struct Register8(u8);

#[allow(non_snake_case)]
pub struct CPU {
    A: Register8,
    X: Register8,
    Y: Register8,
    PC: Register16,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            A: Register8(0), 
            X: Register8(0), 
            Y: Register8(0), 
            PC: Register16(0),
        }
    }

    pub fn run(&self) -> () {
        loop {}
    }
}
