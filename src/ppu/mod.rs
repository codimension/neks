mod memory;
mod register;

use memory::GraphicsMemory;
use register::RegisterBank;

pub struct PPU {
    // PPU Registers - MemoryBus accesses these
    // Therefore visible to CPU through certain memory addresses
    registers: RegisterBank,

    framebuffer: [u8; 1000],

    oam_address: u8,
    oam_data: [u8; 0x100],
    memory: GraphicsMemory,

    cpu_cycles: u16,
    scanline: u16,
    cycles: u16,
}

impl PPU {
    pub fn init() -> Self {
        Self {
            registers: RegisterBank::init(),

            framebuffer: [0; 1000],

            cpu_cycles: 0,

            oam_address: 0,
            oam_data: [0xff; 0x100],
            memory: GraphicsMemory::init(),

            scanline: 0,
            cycles: 0,
        }
    }

    pub fn read_register(&mut self, address: u8) -> u8 {
        match address {
            2 => self.registers.read_status(),
            4 => self.registers.read_sprite_data(),
            7 => self.registers.read_ppu_data(),
            _ => 0, // Invalid read | TODO: Find out if this needs to be handled
        }
    }

    pub fn step(&mut self) {
        let (cpu_cycles, _) = self.cpu_cycles.overflowing_add(1);
        self.cpu_cycles = cpu_cycles;
        self.tick(); self.tick(); self.tick();
        if self.cpu_cycles % 5 == 0 {
            self.tick();
        } // PAL timing: 3.2 PPU ticks for every CPU tick
    }

    fn tick(&mut self) {
        match self.scanline {
            x if x < 240 => {
                match self.cycles {
                    0 => (), // Idle
                    _ => (),
                }
            }
            241 => self.registers.set_vblank(),
            _ => (),
        }
        if self.cycles == 340 {
            self.cycles = 0;
            if self.scanline == 310 {
                self.scanline = 0;
            } 
            else {
                self.scanline += 1;
            }
        }
        else {
            self.cycles += 1;
        }

    }

    pub fn write_register(&mut self, address: u8, value: u8) {
        match address {
            0 => self.registers.write_cr1(value),
            1 => self.registers.write_cr2(value),
            2 => (), // Read-only
            3 => self.registers.write_sprite_address(value),
            4 => self.registers.write_sprite_data(value),
            5 => self.registers.write_ppu_scroll(value),
            6 => self.registers.write_ppu_address(value),
            7 => self.registers.write_ppu_data(value),
            _ => panic!("Invalid write to PPU register!"),
        }
    }


    pub fn write_oam_address(&mut self, address: u8) {
        self.oam_address = address;
    }

    pub fn write_oam_data(&mut self, data: u8) {
        self.oam_data[self.oam_address as usize] = data;
        self.oam_address = self.oam_address.wrapping_add(1);
    }
}

struct Tile {

}

struct Sprite {

}