mod instructions;
mod register;
mod alu;

use std::convert::TryInto;

use crate::ines::Cartridge;
use crate::memory::MemoryBus;

use register::{Flags, RegisterBank};
use instructions::{AddressMode, Instruction};
use instructions::AddressMode::*;
use instructions::Instruction::*;
use alu::*;

#[inline]
fn le_address_16(low: u8, high: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
}

#[allow(non_snake_case)]
pub struct CPU {
    registers: RegisterBank,
    PC: u16,

    memory: MemoryBus,   
    cartridge: Cartridge,

    is_running: bool,

    address_line: u16,

    opcode: u8,
    next_opcode: u8,
}

impl CPU {
    pub fn init(cartridge: Cartridge) -> Self {
        let mut cpu = Self {
            registers: RegisterBank::init(),
            PC: 0,

            memory: MemoryBus::init(),
            cartridge: cartridge,

            is_running: false,

            address_line: 0,

            opcode: 0,
            next_opcode: 0,
        };

        // Load cartridge into locations 0x8000 -> 0xffff
        cpu.memory.load_cartridge(&cpu.cartridge);

        // Location of the so-called reset vector
        let low = cpu.memory.read_byte(0xfffc);
        let high = cpu.memory.read_byte(0xfffd);
        cpu.PC = ((high as u16) << 8) | (low as u16);

        cpu
    }

    pub fn run(&mut self) -> () {
        self.is_running = true;
        println!("A: {}, S: {}, X: {}, Y: {}, P: {:b}",
                self.registers.A,
                self.registers.S,
                self.registers.X,
                self.registers.Y,
                self.registers.P.bits(),
            );
        while self.is_running {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let pc = self.PC;
        self.opcode = self.next();
        // The 6502 always read two bytes at a time.
        self.next_opcode = self.next();
        let instruction = instructions::decode(self.opcode);
        //println!("{:x}: {:x} {:x}> {:?}", pc, self.opcode, self.next_opcode, instruction);
        self.execute(instruction);
        /*
        println!("A: {}, S: {}, X: {}, Y: {}, P: {:b}",
                self.registers.A,
                self.registers.S,
                self.registers.X,
                self.registers.Y,
                self.registers.P.bits(),
            );
        */
    }

    #[inline]
    /// For one byte instructions, set the counter back
    /// The actual NES CPU always fetches the next instruction, but for the one byte instructions,
    /// doesn't throw it away
    /// When we come to timing, I might change how this works (to require 2 byte instructions that don't use
    /// the operand to explicitly throw it away)
    fn prev(&mut self) -> () {
        self.PC -= 1;
    }

    #[inline]
    fn next(&mut self) -> u8 {
        let ret = self.memory.read(self.PC);
        self.PC += 1;
        ret
    }

    // Handles common timing logic, fetching and storing of results,
    // without maintaining any extra state
    fn rmw<F>(&mut self, address_mode: AddressMode, mut action: F)
        where F: FnMut(&mut CPU, u8) -> u8 
    {
        let input = self.load_input(address_mode);
        let value = action(self, input);
        match address_mode {
            Accumulator => self.registers.A = value,
            _ => self.memory.write(self.address_line, value),
        }
    }

    #[inline]
    fn load_input(&mut self, address_mode: AddressMode) -> u8 {
        match address_mode {
            Immediate => {
                self.next_opcode
            },              
            Accumulator => {
                self.registers.A
            },
            _=> {
                self.calculate_address(address_mode);
                self.memory.read(self.address_line)
            }
        }
    }

    #[inline]
    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            ADC(address_mode) => {
                let carry_in = self.registers.P.contains(Flags::C);
                let (value, carry, overflow) = alu::adc(self.registers.A, self.load_input(address_mode), carry_in);
                self.registers.A = value;
                self.update_flags_ZNCV(value, carry, overflow);
            },
            AND(address_mode) => {               
                let value = alu::and(self.registers.A, self.load_input(address_mode));
                self.registers.A = value;
                self.update_flags_ZN(value);
            },
            ASL(address_mode) => {
                let (value, carry) = asl(self.load_input(address_mode));
                self.update_flags_ZNC(value, carry);
                match address_mode {
                    Accumulator => self.registers.A = value,
                    _ => self.memory.write(self.address_line, value),
                }
            },
            BCC => {
                if !self.registers.P.contains(Flags::C) {
                    self.branch();
                }
            },
            BCS => {
                if self.registers.P.contains(Flags::C) {
                    self.branch();
                }
            }
            BEQ => {
                if self.registers.P.contains(Flags::Z) {
                    self.branch();
                }
            },
            BIT(address_mode) => {
                // Do an and between A and the contents of memory
                // The idea is that A contains a mask
                // Then the zero flags tells you if the bit was not set
                // So zero flag not being set tells an application that the bit _was_ set
                // We then set the V and C flags depending on the values in bits 6 and 7 of the memory value

                let input = self.load_input(address_mode);
                let value = alu::and(self.registers.A, self.load_input(address_mode));
                self.registers.P.remove(Flags::V | Flags::C);
                let memory_value = Flags::from(input) & (Flags::V | Flags::C);
                self.registers.P.insert(memory_value);
                self.update_flags_ZN(value);
            },
            BMI => {
                if self.registers.P.contains(Flags::N) {
                    self.branch();
                }
            },
            BNE => {
                if !self.registers.P.contains(Flags::Z) {
                    self.branch();
                }
            },
            BPL => {
                if !self.registers.P.contains(Flags::N) {
                    self.branch();
                }
            },
            BRK => {
                self.push(self.registers.P.into());
                self.push16(self.PC);
                let low = self.memory.read(0xfffe_u16);
                let high = self.memory.read(0xffff_u16);
                self.PC = ((high as u16) << 8) & (low as u16);
                self.registers.P.insert(Flags::B);
            },
            BVC => {
                if !self.registers.P.contains(Flags::V) {
                    self.branch();
                }
            },
            BVS => {
                if self.registers.P.contains(Flags::V) {
                    self.branch();
                }
            },
            CLR(flag) => {
                self.registers.P.remove(flag);
                self.prev(); // One byte instruction
            },
            CMP(address_mode) => {
                let (value, carry) = alu::cmp(self.registers.A, self.load_input(address_mode));
                self.update_flags_ZNC(value, carry);
            },
            CPX(address_mode) => {
                let (value, carry) = alu::cmp(self.registers.X, self.load_input(address_mode));
                self.update_flags_ZNC(value, carry);
            },
            CPY(address_mode) => {
                let (value, carry) = alu::cmp(self.registers.Y, self.load_input(address_mode));
                self.update_flags_ZNC(value, carry);
            },
            DEC(address_mode) => {
                let value = alu::dec(self.load_input(address_mode));
                self.update_flags_ZN(value);
                self.memory.write(self.address_line, value);
                self.prev();
            },
            DEX => {
                let value = alu::dec(self.registers.X);
                self.update_flags_ZN(value);
                self.memory.write(self.address_line, value);
                self.prev();
            },
            DEY => {
                let value = alu::dec(self.registers.Y);
                self.update_flags_ZN(value);
                self.memory.write(self.address_line, value);
                self.prev();
            },
            EOR(address_mode) => {
                let value = alu::eor(self.registers.A, self.load_input(address_mode));
                self.update_flags_ZN(value);
                self.registers.A = value;
            },
            INC(address_mode) => {
                let value = alu::inc(self.load_input(address_mode));
                self.update_flags_ZN(value);
                self.memory.write(self.address_line, value);
                self.prev();
            },
            INX => {
                let value = alu::inc(self.registers.X);
                self.update_flags_ZN(value);
                self.memory.write(self.address_line, value);
                self.prev();
            },
            INY => {
                let value = alu::inc(self.registers.Y);
                self.update_flags_ZN(value);
                self.memory.write(self.address_line, value);
                self.prev();
            },
            JMP(address_mode) => self.jmp(address_mode),
            JSR => {
                // It's important here to read the jump address from JSR first so that the PC is then 
                // correctly pointing at the return address
                self.calculate_address(Absolute);
                self.push16(self.PC);
                self.PC = self.address_line;
            },
            LDA(address_mode) => {
                self.registers.A = self.load(address_mode);
            },
            LDX(address_mode) => {
                self.registers.X = self.load(address_mode);
            },
            LDY(address_mode) => {
                self.registers.Y = self.load(address_mode);
            },
            LSR(address_mode) => {
                let (value, carry) = alu::lsr(self.load_input(address_mode));
                self.update_flags_ZNC(value, carry);
                match address_mode {
                    Accumulator => self.registers.A = value,
                    _ => self.memory.write(self.address_line, value),
                }
            },
            ORA(address_mode) => {
                let value = alu::or(self.registers.A, self.load_input(address_mode));
                self.update_flags_ZN(value);
                self.registers.A = value;
            },
            NOP => {

            },
            PHA => {
                self.push(self.registers.A);
            },
            PHP => {
                self.push(u8::from(self.registers.P));
            },
            PLA => {
                self.registers.A = self.pull();
            },
            PLP => {
                self.registers.P = Flags::from(self.pull());
            },
            ROL(address_mode) => {
                let carry_in = self.registers.P.contains(Flags::C);
                let (value, carry) = alu::rol(self.load_input(address_mode), carry_in);
                self.update_flags_ZNC(value, carry);

                match address_mode {
                    Accumulator => self.registers.A = value,
                    _ => self.memory.write(self.address_line, value),
                }
            },
            ROR(address_mode) => {
                let carry_in = self.registers.P.contains(Flags::C);
                let (value, carry) = alu::ror(self.load_input(address_mode), carry_in);
                self.update_flags_ZNC(value, carry);

                match address_mode {
                    Accumulator => self.registers.A = value,
                    _ => self.memory.write(self.address_line, value),
                }
            },
            RTI => {
                self.registers.P = Flags::from(self.pull());
                self.PC = self.pull16();
            },
            RTS => {
                // TODO: Check if I'm doing this right.
                // In what order does my fetch cycle pull the PC and increment it?
                self.PC = self.pull16();
            },
            SBC(address_mode) => {
                let carry_in = self.registers.P.contains(Flags::C);
                let (value, carry, overflow) = alu::sbc(self.registers.A, self.load_input(address_mode), carry_in);
                self.registers.A = value;
                self.update_flags_ZNCV(value, carry, overflow);
            },
            SET(flag) => {
                self.registers.P.insert(flag);
                self.prev(); // One byte instruction
            },
            STA(address_mode) => {
                self.store(address_mode, self.registers.A);
            },
            STX(address_mode) => {
                self.store(address_mode, self.registers.X);
            },
            STY(address_mode) => {
                self.store(address_mode, self.registers.Y);
            },
            TAX => {
                self.registers.X = self.registers.A;
                self.prev(); // One byte instruction
            },
            TAY => {
                self.registers.Y = self.registers.A;
                self.prev(); // One byte instruction
            },
            TSX => {
                self.registers.X = self.registers.S;
                self.prev(); // One byte instruction
            },
            TXA => {
                self.registers.A = self.registers.X;
                self.prev(); // One byte instruction
            },
            TXS => {
                self.registers.S = self.registers.X;
                self.prev(); // One byte instruction
            },
            TYA => {
                self.registers.A = self.registers.Y;
                self.prev(); // One byte instruction
            }
        }
    }

    #[inline]
    fn jmp(&mut self, address_mode: AddressMode) {
        match address_mode {
            Absolute | Indirect => {
                self.calculate_address(address_mode);
            },
            _ => panic!("Internal processor error: Tried to jump with incorrect AddressMode"),
        }
        self.PC = self.address_line;
    }

    fn branch(&mut self) {
        self.calculate_address(Relative);
        self.PC = self.address_line;
    }

    #[inline]
    fn push(&mut self, value: u8) {
        self.memory.write(self.registers.S as u16, value);
        self.registers.S = alu::dec(self.registers.S);
    }

    fn push16(&mut self, value: u16) {
        self.memory.write(self.registers.S as u16, (value >> 8).try_into().unwrap());
        self.registers.S = alu::dec(self.registers.S);
        self.memory.write(self.registers.S as u16, (value & 0xff).try_into().unwrap());
        self.registers.S = alu::dec(self.registers.S);
    }

    #[inline]
    fn pull(&mut self) -> u8 {
        self.registers.S = alu::inc(self.registers.S);
        self.memory.read(self.registers.S as u16)
    }

    fn pull16(&mut self) -> u16 {
        self.registers.S = alu::inc(self.registers.S);
        let low = self.memory.read(self.registers.S as u16);
        self.registers.S = alu::inc(self.registers.S);
        let high = self.memory.read(self.registers.S as u16);
        ((high as u16) << 8) | (low as u16)
    }

    #[inline]
    fn store(&mut self, address_mode: AddressMode, value: u8) {

        match address_mode {
            Accumulator => self.registers.A = value,
            Immediate   => panic!("Attempt to store to an immediate operand!"),
            _ => {        
                self.calculate_address(address_mode);
                self.memory.write(self.address_line, value)
            },
        }
    }

    #[inline]
    fn load(&mut self, address_mode: AddressMode) -> u8 {
        let value = match address_mode {
            Accumulator => self.registers.A,
            Immediate   => self.next_opcode,
            _ => {        
                self.calculate_address(address_mode);
                self.memory.read(self.address_line)
            },
        };
        self.update_flags_ZN(value);
        value
    }

    #[inline]
    #[allow(non_snake_case)]
    fn update_flags_ZNCV(&mut self, value: u8, carry: bool, overflow: bool) {
        self.update_flags_ZNC(value, carry);
        self.registers.P.set(Flags::V, overflow);
    }

    #[inline]
    #[allow(non_snake_case)]
    fn update_flags_ZNC(&mut self, value: u8, carry: bool) {
        self.update_flags_ZN(value);
        self.registers.P.set(Flags::C, carry);
    }
    
    #[inline]
    #[allow(non_snake_case)]
    fn update_flags_ZN(&mut self, value: u8) {
        self.registers.P.set(Flags::Z, value == 0);
        self.registers.P.set(Flags::N, value & 0x80 == 0x80);
    }

    #[inline]
    fn calculate_address(&mut self, address_mode: AddressMode) {
        match address_mode {
            Accumulator => {
                panic!("Tried to calculate memory address for an accumulator instruction")
            },
            Immediate => {
                // The value to use is the operand, no memory load is required
                panic!("Tried to calculate memory address for an immediate operand")
            },
            ZeroPage => {
                // The operand for a Zero Page instruction gives the LSB of the memory location to load
                // The MSB is zero, so that the memory location loaded is in the 'zero page'
                self.address_line = self.next_opcode as u16;
            },
            ZeroPageX => {
                // Take the zero page address given by the operand and add the value of the X register to it
                // This result of this calculation is wrapping, so that if the value is greater than 0xff then
                // any bits in the most significant byte will be zeroised so that the address remains a zero page one
                self.memory.read(self.next_opcode as u16); // Dummy read
                self.address_line = self.next_opcode.wrapping_add(self.registers.X) as u16;
            },
            ZeroPageY => {
                // Take the zero page address given by the operand and add the value of the Y register to it
                // This result of this calculation is wrapping, so that if the value is greater than 0xff then
                // any bits in the most significant byte will be zeroised so that the address remains a zero page one
                self.memory.read(self.next_opcode as u16); // Dummy read
                self.address_line = self.next_opcode.wrapping_add(self.registers.Y) as u16;
            },
            Relative => {
                let (low, carry, _) = alu::adc(self.next_opcode, (self.PC & 0xff) as u8, false);
                // Sign extend the 8 bit signed offset
                let a = match self.next_opcode & 0x80 {
                    0x80 => 0xff,
                    _    => 0x00
                };
                let (high, _, _) = alu::adc(a, (self.PC >> 8) as u8, carry);
                self.address_line = le_address_16(low, high);
            },
            Absolute => {
                // Operand is a 16-bit instruction

                // Little-endian read
                let low = self.next_opcode; let high = self.next();
                self.address_line = le_address_16(low, high);
            },
            AbsoluteX => {
                // Operand is a 16-bit address. The value of X is then added to the operand to get effective address
                // What actually appears to take place is that the low byte of operand is added to X
                // Then the memory location implied by the 8-bit result of this and the high byte of the operand
                // is read from.
                // If there was a carry when adding X, the processor then applies the carry and reads from the correct
                // address as calculated by the 16 bit addition of HHLL + XX, where HH and LL are the high and low bytes
                // of the operand and XX is the contents of the X register. 
                // So when a page boundary is crossed, effectively this takes an extra cycle.

                // Little-endian read
                let low = self.next_opcode; let high = self.next();
                let (new_low, carry, _) = alu::adc(low, self.registers.X, false);
                let first_location = le_address_16(new_low, high);
                self.memory.read(first_location);
                let (new_high, _, _) = alu::adc(high, 0, carry);
                self.address_line = le_address_16(new_low, new_high);
            },
            AbsoluteY => {
                // Exactly like (Absolute,X) addressing, but uses the Y register as the index
                let low = self.next_opcode; let high = self.next();
                let (new_low, carry, _) = alu::adc(low, self.registers.Y, false);
                let first_location = le_address_16(new_low, high);
                self.memory.read(first_location);
                let (new_high, _, _) = alu::adc(high, 0, carry);
                self.address_line = le_address_16(new_low, new_high);
            },
            Indirect => {
                // The immediate 16-bit operand points to the memory location of the effective address!
                let low = self.next_opcode; let high = self.next();
                let indirect = le_address_16(low, high);
                let low = self.memory.read(indirect);
                let high = self.memory.read(indirect.wrapping_add(1));
                self.address_line = le_address_16(low, high);
            },
            XIndirect => {
                // Take an 8-bit operand and add to the X-register, discarding the carry
                // This is a pointer to the zero page
                // It points to the LSB of the effective address, and the next location in the zero page points to 
                // the MSB of the effective address
                let indirect = self.next_opcode;
                self.memory.read(indirect as u16); //TODO: Really not sure on this. Only one source mentions it.
                let pointer = indirect.wrapping_add(self.registers.X) as u16;
                let low = self.memory.read(pointer);
                let high = self.memory.read(pointer.wrapping_add(1));
                self.address_line = le_address_16(low, high);
            },
            IndirectY => {
                // The 8-bit operand points to a location in the zero page.
                // The value of the memory location pointed to is added to the Y register.
                // The result of this is the low byte of the effective address
                // The next value in the zero page is added to the carry from the previous addition
                // to give the high byte of the effective address
                //
                // So essentially the effective address is the 16-bit number pointed to in the zero page
                // added to the Y register.
                let pointer = self.next_opcode as u16;
                let low = self.memory.read(pointer);
                let high = self.memory.read(pointer.wrapping_add(1));
                let (new_low, carry, _) = alu::adc(low, self.registers.Y, false);
                self.memory.read(le_address_16(new_low, high));
                let (new_high, _, _) = alu::adc(high, 0, carry);
                self.address_line = le_address_16(new_low, new_high);
            },
        }
    }
}
