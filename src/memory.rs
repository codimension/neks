/// A representation of the CPU's access to memory
pub(crate) struct MemoryBus {
    /// 2kb of RAM
    memory: [u8; 0xFFFF],
}

impl MemoryBus {
    pub fn init() -> Self {
        Self {
            memory: [0; 0xFFFF],
        }
    }
    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }
}