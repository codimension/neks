use crate::ines::Cartridge;
use crate::ppu::PPU;

/// A representation of the CPU's access to memory
pub(crate) struct MemoryBus {
    /// 2kb of RAM
    memory: [u8; 2048],
    rom_data: [u8; 32768],
    ppu: PPU,
}

impl MemoryBus {
    pub fn init() -> Self {
        Self {
            memory: [0; 2048],
            rom_data: [0; 32768],
            ppu: PPU::init(),
        }
    }

    pub fn read_byte(&mut self, address: u16) -> u8 { // can change PPU
        if address < 0x800 {
            self.memory[address as usize]
        }
        else if address >= 0x800 && address < 0x1000 {
            self.memory[address as usize - 0x800]
        }
        else if address >= 0x1000 && address < 0x1800 {
            self.memory[address as usize - 0x1000]
        }
        else if address >= 0x1800 && address < 0x2000 {
            self.memory[address as usize - 0x1800]
        }
        else if address == 0x2002 {
            self.ppu.read_status()
        }
        else if address == 0x2004 {
            self.ppu.read_sprite_data()
        }
        else if address == 0x2007 {
            self.ppu.read_ppu_data()
        }
        else if address >= 0x2008 && address < 0x4000 {
            self.read_byte(0x2000 + ((address - 0x2000) % 8))
        }
        else { // address >= 0x8000
            // If I add support for carts with mappers in future, will move the read logic to the cartridge
            self.rom_data[address as usize - 0x8000]
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        if address < 0x800 {
            self.memory[address as usize] = value;
        }
        else if address >= 0x800 && address < 0x1000 {
            self.memory[address as usize - 0x800] = value;
        }
        else if address >= 0x1000 && address < 0x1800 {
            self.memory[address as usize - 0x1000] = value;
        }
        else if address >= 0x1800 && address < 0x2000 {
            self.memory[address as usize - 0x1800] = value;
        }
        else if address >= 0x2000 && address < 0x2008 {
            // Handle writing to PPU registers
            match address {
                0x2000 => self.ppu.write_cr1(value),
                0x2001 => self.ppu.write_cr2(value),
                0x2002 => (), // Read-only
                0x2003 => self.ppu.write_sprite_address(value),
                0x2004 => self.ppu.write_sprite_data(value),
                0x2005 => self.ppu.write_ppu_scroll(value),
                0x2006 => self.ppu.write_ppu_address(value),
                0x2007 => self.ppu.write_ppu_data(value),
                _ => (), // Not reachable
            }
        }
        else if address >= 0x2008 && address < 0x4000 {
            self.write_byte(0x2000 + ((address - 0x2000) % 8), value);
        }
        else { // address >= 0x8000
            // If I add support for carts with mappers in future, will move the read logic to the cartridge
            self.rom_data[address as usize - 0x8000] = value;
        }
    }

    pub fn load_cartridge(&mut self, cartridge: &Cartridge) {
        let dest = &mut self.rom_data;
        // Note upper bound one higher than largest index, because it's just how ranges work
        let rom  = &cartridge.prg_rom_data[0x0000..0x8000]; 
        dest.copy_from_slice(rom);
    }
}