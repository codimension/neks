use std::convert::Into;

use crate::ines::Cartridge;
use crate::ppu::PPU;

const OAMADDR: u16 = 0x2003;

/// A representation of the CPU's access to memory
pub(crate) struct MemoryBus {
    /// 2kb of RAM
    memory: [u8; 2048],
    rom_data: [u8; 32768],
    ppu: PPU,
    cycles: u8,
}

impl MemoryBus {
    pub fn init() -> Self {
        Self {
            memory: [0; 2048],
            rom_data: [0; 32768],
            ppu: PPU::init(),
            cycles: 0,
        }
    }

    pub fn read<T: Into<u16>>(&mut self, address: T) -> u8 {
        let result = self.read_byte(address.into());
        self.tick();
        result
    }

    pub fn write<T: Into<u16>>(&mut self, address: T, value: u8) {
        self.write_byte(address.into(), value);
        self.tick();
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        // Match syntax is much neater than ifs, but unfortunately exclusive ranges
        // (low <= x < high) are feature-gated, and so only live on nightly
        match address {
            0x000..=0x7ff   => self.memory[address as usize],
            0x800..=0xfff   => self.memory[address as usize - 0x800],
            0x1000..=0x17ff => self.memory[address as usize - 0x1000],
            0x1800..=0x1fff => self.memory[address as usize - 0x1800],
            // There are 8 memory-mapped PPU registers, and these are mirrored for the next block
            // Since only 8 values, only the first 3 bits matter, so mask it and provide it to the PPU
            0x2000..=0x3fff => self.ppu.read_register((address & 0x7) as u8),
            0x8000..=0xffff => self.rom_data[address as usize - 0x8000],
            _ => panic!(""), // TODO: Do we need to return a Result?
                             // I don't know if any NES programs ever attempt to read non-valid memory addresses
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x000..=0x7ff   => self.memory[address as usize] = value,
            0x800..=0xfff   => self.memory[address as usize - 0x800] = value,
            0x1000..=0x17ff => self.memory[address as usize - 0x1000] = value,
            0x1800..=0x1fff => self.memory[address as usize - 0x1800] = value,
            // There are 8 memory-mapped PPU registers, and these are mirrored for the next block
            // Since only 8 values, only the first nibble matters, so mask it and provide it to the PPU
            0x2000..=0x3fff => self.ppu.write_register((address & 0xf) as u8, value),
            // Mirrors of 0x2000..0x2007
            0x4014 => self.write_dma(value),
            0x8000..=0xffff => self.rom_data[address as usize - 0x8000] = value, 
            _ => (), // If memory isn't mapped, do nothing
        }
    }
    
    fn write_dma(&mut self, offset: u8) {
        // From NESDEV
        // Not counting the OAMDMA write tick, the above procedure takes 513 CPU cycles (+1 on odd CPU cycles): 
        // first one (or two) idle cycles, and then 256 pairs of alternating read/write cycles. 
        // The OAMDMA write tick is taken care of when the CPU calls 'write'.
        if self.cycles % 2 == 1 {
            self.tick();
        }
        for i in 1..256 {
            let data = self.read_byte(0x100 * (offset as u16) + i);
            self.tick();
            self.ppu.write_oam_data(data);
            self.tick();
        }
    }

    pub fn load_cartridge(&mut self, cartridge: &Cartridge) {
        let dest = &mut self.rom_data;
        // Note upper bound one higher than largest index, because it's just how ranges work
        let rom  = &cartridge.prg_rom_data[0x0000..0x8000]; 
        dest.copy_from_slice(rom);
    }

    fn tick(&mut self) {
        let (cycles, _) = self.cycles.overflowing_add(1);
        self.cycles = cycles;
        self.ppu.step();
    }
}