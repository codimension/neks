use crate::ines::Cartridge;

pub mod registers;
pub use registers::*;

#[allow(non_snake_case)]
pub struct CPU {
    A: Register8,
    X: Register8,
    Y: Register8,
    PC: Register16,
    cartridge: Cartridge,

    is_running: bool,
}

impl CPU {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            A: Register8::new(0), 
            X: Register8::new(0), 
            Y: Register8::new(0), 
            PC: Register16::new(0),
            cartridge: cartridge,
            is_running: false,
        }
    }

    pub fn execute(&mut self, instruction: u8) {
        match instruction {
            0x00 => println!("break"),
            0x01 => println!("ora izx 6"),
            _ => println!("Unimplemented"),
        }
    }

    pub fn run(&mut self) -> () {
        self.is_running = true;
        while self.is_running {
            let instruction = self.consume_byte();
            println!("Opcode: {}", instruction);
            self.execute(instruction);
        }
    }

    pub fn peek_byte(&mut self) -> u8 {
        self.cartridge[self.PC]
    }

    pub fn consume_byte(&mut self) -> u8 {
        let val = self.cartridge[self.PC];
        self.PC.increment();
        val
    }
}
