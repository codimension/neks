use bitflags::*;

bitflags! {
    struct ControlRegister1: u8 {
        const BASE_TABLE = 0b00000011;
        const VRAM_INC = 0b00000100;
        const BACKGROUND_PATTERN = 0b00001000;
        const SPRITE_PATTERN = 0b00010000;
        const SPRITE_SIZE = 0b00100000;
        const NMI_INTERRUPTS = 0b10000000;
    }
}

bitflags! {
    struct ControlRegister2: u8 {
        const BW = 0b00000001;
        const BACKGROUND_CLIPPING = 0b00000010;
        const SPRITE_CLIPPING = 0b000000100;
        const BACKGROUND_RENDERING = 0b00010000;
        const SPRITE_RENDERING = 0b00100000;
        const INTENSIFY_RED = 0b00100000;
        const INTENSIFY_GREEN = 0b01000000;
        const INTENSIFY_BLUE = 0b10000000;
    }
}

bitflags! {
    struct StatusRegister: u8 {
        const SPRITE0 = 0b00100000;
        const MAX_SPRITES_SCANLINE = 0b01000000;
        const VBLANK = 0b10000000;
    }
}


pub struct PPU {
    // PPU Registers - MemoryBus accesses these
    // Therefore visible to CPU through certain memory addresses
    cr1: ControlRegister1,
    cr2: ControlRegister2,
    status: StatusRegister,
    sprite_address: u8,
    sprite_data: u8,
    ppu_scroll: u8,
    ppu_address: u8,
    ppu_data: u8,
}

impl PPU {
    pub fn init() -> Self {
        Self {
            cr1: ControlRegister1::from_bits_truncate(0),
            cr2: ControlRegister2::from_bits_truncate(0),
            status: StatusRegister::from_bits_truncate(0),
            sprite_address: 0,
            sprite_data: 0,
            ppu_scroll: 0,
            ppu_address: 0,
            ppu_data: 0,
        }
    }

    pub fn read_status(&mut self) -> u8 {
        // When this is read, the Vblank bit is set to 0
        self.status.remove(StatusRegister::VBLANK);
        self.status.bits
    }
    pub fn read_sprite_data(&self) -> u8 {
        self.sprite_data
    }
    pub fn read_ppu_data(&self) -> u8 {
        self.ppu_data
    }

    pub fn write_cr1(&mut self, bits: u8) {
        self.cr1 = ControlRegister1::from_bits_truncate(bits);
    }
    pub fn write_cr2(&mut self, bits: u8) {
        self.cr2 = ControlRegister2::from_bits_truncate(bits);
    }
    pub fn write_sprite_address(&mut self, bits: u8) {
        self.sprite_address = bits;
    }
    pub fn write_sprite_data(&mut self, bits: u8) {
        self.sprite_data = bits;
    }
    pub fn write_ppu_scroll(&mut self, bits: u8) {
        self.ppu_scroll = bits;
    }
    pub fn write_ppu_address(&mut self, bits: u8) {
        self.ppu_address = bits;
    }
    pub fn write_ppu_data(&mut self, bits: u8) {
        self.ppu_data = bits;
    }
}