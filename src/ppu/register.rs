use bitflags::*;

const FINE_Y: u16 = 0b_0111_00_00000_00000;
const COARSE_X: u16 = 0b_0000_00_00000_11111;
const COARSE_Y: u16 = 0b_0000_00_11111_00000;
const NAMETABLE: u16 = 0b_0000_11_00000_00000;
const FINE_Y_COMPLEMENT: u16 = FINE_Y ^ 0xffff;
const COARSE_X_COMPLEMENT: u16 = COARSE_X ^ 0xffff;
const COARSE_Y_COMPLEMENT: u16 = COARSE_Y ^ 0xffff;
const NAMETABLE_COMPLEMENT: u16 = NAMETABLE ^ 0xffff;

// The internal registers of the PPU control the operation of scrolling.
// These registers are 15 bits, but to emulate we're going to store the data
// in an easier-to-access format, and serialize/deserialize to u16
struct InternalRegister {
    coarse_x: u8,
    y: u8,
    nametable: u8,
}

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

impl InternalRegister {

    pub fn init() -> Self {
        Self {
            coarse_x: 0,
            y: 0,
            nametable: 0,
        }
    }

    pub fn set_low_address(&mut self, value: u8) {
        self.coarse_x = value & 0b0001_1111;
        let y = (value & 0b1110_0000) >> 2;
        self.y &= 0b1100_0111;
        self.y |= y;
    }
    pub fn set_high_address(&mut self, value: u8) {
        let coarse_y = value & 0b0011;
        let fine_y = (value & 0b0111_0000) >> 4;
        self.y &= 0b0011_1000;
        self.y |= fine_y | (coarse_y << 6);
    }

    pub fn write_x_scroll(&mut self, value: u8) {
        self.coarse_x = (value & 0b1111_1000) >> 3;
    }

    pub fn write_y_scroll(&mut self, value: u8) {
        self.y = value;
    }

    pub fn increment_coarse_x(&mut self) {
        if self.coarse_x == 31 {
            self.coarse_x = 0;
            self.nametable ^= 1;
        }
        else {
            self.coarse_x += 0b01;
        }
    }

    pub fn increment_y(&mut self) {
        match self.y {
            // Coarse y = 29, fine y = 7
            0b1110_1111 => {
                self.y = 0;
                self.nametable ^= 0b10;
            },
            // 8 bit overflow, nametable doesn't get flipped
            // Can occur if a value greater than 29 for y is written
            0b1111_1111 => self.y = 0,
            _ => self.y += 1,
        }
    }

}

pub struct RegisterBank {
    cr1: ControlRegister1,
    cr2: ControlRegister2,
    status: StatusRegister,
    sprite_address: u8,
    sprite_data: u8,
    ppu_data: u8,

    first_write: bool,
    fine_x_scroll: u8,
    t: InternalRegister,
    v: InternalRegister,
}

impl RegisterBank {
    pub fn init() -> Self {
        Self {
            cr1: ControlRegister1::from_bits_truncate(0),
            cr2: ControlRegister2::from_bits_truncate(0),
            status: StatusRegister::from_bits_truncate(0),
            sprite_address: 0,
            sprite_data: 0,
            ppu_data: 0,

            fine_x_scroll: 0,
            first_write: true,
            t: InternalRegister::init(),
            v: InternalRegister::init(),
        }
    }

    pub fn read_status(&mut self) -> u8 {
        // When this is read, the Vblank bit is set to 0
        let value = self.status.bits;
        self.first_write = true;
        self.status.remove(StatusRegister::VBLANK);
        value
    }
    pub fn read_sprite_data(&self) -> u8 {
        self.sprite_data
    }
    pub fn read_ppu_data(&self) -> u8 {
        self.ppu_data
    }

    pub fn set_vblank(&mut self) {
        self.status.insert(StatusRegister::VBLANK);
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
        match self.first_write {
            true =>  {
                self.t.write_x_scroll(bits);
                self.fine_x_scroll = bits & 0b0000_0111;
            }
            false => self.t.write_y_scroll(bits),
        }
        self.first_write = self.first_write && false;
    }
    pub fn write_ppu_address(&mut self, bits: u8) {
        match self.first_write {
            true => self.t.set_high_address(bits),
            false => self.t.set_low_address(bits),
        }
        self.first_write = self.first_write && false;
    }
    pub fn write_ppu_data(&mut self, bits: u8) {
        self.ppu_data = bits;
    }
}

