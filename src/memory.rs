use crate::ines::Cartridge;

/// A representation of the CPU's access to memory
pub(crate) struct MemoryBus {
    /// 2kb of RAM
    memory: [u8; 0x10000],
}

impl MemoryBus {
    pub fn init() -> Self {
        Self {
            memory: [0; 0x10000],
        }
    }
    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    pub fn load_cartridge(&mut self, cartridge: &Cartridge) {
        let dest = &mut self.memory[0x8000..0xffff];
        let rom  = &cartridge.prg_rom_data[0x0000..0x7fff];
        dest.copy_from_slice(rom);
    }
}