mod instructions;
mod register;
mod alu;

use std::convert::TryInto;
use std::rc::Rc;
use std::cell::RefCell;

use crate::ines::Cartridge;
use crate::memory::MemoryBus;
use crate::ppu::PPU;


use register::{Flags, Registers};
use instructions::{AddressMode, Instruction};
use instructions::AddressMode::*;
use instructions::Instruction::*;
use alu::ALU;

#[allow(non_snake_case)]
pub struct CPU {
    registers: Registers,
    PC: u16,

    memory: MemoryBus,   
    cartridge: Cartridge,
    alu: ALU,

    clock: u16,
    address_line: u16,
    is_running: bool,

    opcode: u8,
    next_opcode: u8,
}

impl CPU {
    pub fn init(cartridge: Cartridge, ppu: Rc<RefCell<PPU>>) -> Self {
        let mut cpu = Self {
            registers: Registers::init(),
            PC: 0,

            clock: 0,
            memory: MemoryBus::init(ppu),
            cartridge: cartridge,
            alu: ALU::new(),

            address_line: 0,
            is_running: false,

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
        println!("A: {}, S: {}, X: {}, Y: {},",
                self.registers.A,
                self.registers.S,
                self.registers.X,
                self.registers.Y
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
        println!("{:x}: {:x} {:x}> {:?}", pc, self.opcode, self.next_opcode, instruction);
        self.execute(instruction);
        println!("A: {}, S: {}, X: {}, Y: {},",
            self.registers.A,
            self.registers.S,
            self.registers.X,
            self.registers.Y
        );
    }


    #[inline]
    pub fn tick(&mut self) {
        self.clock += 1;
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
        let ret = self.memory.read_byte(self.PC);
        self.PC += 1;
        ret
    }

    #[inline]
    fn do_accumulator_arithmetic(&mut self, op: alu::Op) {
        // Assumes that alu.input has been set
        self.alu.input_a = self.registers.A;
        self.alu.flags = self.registers.P;
        
        let (output, flags) = self.alu.do_arithmetic(op);

        self.registers.A = output;
        self.registers.P = flags;
    }

    #[inline]
    fn load_alu_input_b(&mut self, address_mode: AddressMode) {
        match address_mode {
            Immediate => {
                self.alu.input_b = self.next_opcode;
            },              
            Accumulator => {
                self.alu.input_b = self.registers.A;
            }
            _=> {
                self.calculate_address(address_mode);
                self.alu.input_b = self.memory.read_byte(self.address_line);
            }
        }
    }

    #[inline]
    fn load_alu_input_a(&mut self, address_mode: AddressMode) {
        match address_mode {
            Immediate => {
                self.alu.input_a = self.next_opcode;
            },              
            Accumulator => {
                self.alu.input_a = self.registers.A;
            },
            _=> {
                self.calculate_address(address_mode);
                self.alu.input_a = self.memory.read_byte(self.address_line);
            }
        }
    }

    #[inline]
    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            ADC(address_mode) => {
                self.load_alu_input_b(address_mode);
                self.do_accumulator_arithmetic(alu::Op::Adc);
            },
            AND(address_mode) => {
                self.load_alu_input_b(address_mode);
                self.do_accumulator_arithmetic(alu::Op::And);
            },
            ASL(address_mode) => {
                self.load_alu_input_a(address_mode);
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Asl);
                match address_mode {
                    Accumulator => self.registers.A = value,
                    _ => self.memory.write_byte(self.address_line, value),
                }
                self.registers.P = flags;
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
                self.alu.input_a = self.registers.A;
                self.load_alu_input_b(address_mode);
                self.alu.do_arithmetic(alu::Op::And);
                let memory_value = Flags::from(self.alu.input_b) & (Flags::V | Flags::C);
                self.registers.P.insert(memory_value);
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
                let low = self.memory.read_byte(0xfffe);
                let high = self.memory.read_byte(0xffff);
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
                self.alu.input_a = self.registers.A;
                self.load_alu_input_b(address_mode);
                let (_, flags) = self.alu.do_arithmetic(alu::Op::Cmp);
                self.registers.P = flags;
            },
            CPX(address_mode) => {
                self.alu.input_a = self.registers.X;
                self.load_alu_input_b(address_mode);
                let (_, flags) = self.alu.do_arithmetic(alu::Op::Cmp);
                self.registers.P = flags;
            },
            CPY(address_mode) => {
                self.alu.input_a = self.registers.Y;
                self.load_alu_input_b(address_mode);
                let (_, flags) = self.alu.do_arithmetic(alu::Op::Cmp);
                self.registers.P = flags;
            },
            DEC(address_mode) => {
                self.load_alu_input_a(address_mode);
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Dec);
                self.registers.P = flags;
                self.memory.write_byte(self.address_line, value);
                self.prev();
            },
            DEX => {
                self.alu.input_a = self.registers.X;
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Dec);
                self.registers.P = flags;
                self.registers.X = value;
                self.prev();
            },
            DEY => {
                self.alu.input_a = self.registers.Y;
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Dec);
                self.registers.P = flags;
                self.registers.Y = value;
                self.prev();
            },
            EOR(address_mode) => {
                self.load_alu_input_b(address_mode);
                self.do_accumulator_arithmetic(alu::Op::Eor);
            },
            INC(address_mode) => {
                //TODO: need to store
                self.load_alu_input_a(address_mode);
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Inc);
                self.registers.P = flags;
                self.memory.write_byte(self.address_line, value);
                self.prev();
            },
            INX => {
                self.alu.input_a = self.registers.X;
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Inc);
                self.registers.P = flags;
                self.registers.X = value;
                self.prev();
            },
            INY => {
                self.alu.input_a = self.registers.Y;
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Inc);
                self.registers.P = flags;
                self.registers.Y = value;
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
                self.load_alu_input_a(address_mode);
                self.alu.do_arithmetic(alu::Op::Lsr);
                match address_mode {
                    Accumulator => self.registers.A = self.alu.output,
                    _ => self.memory.write_byte(self.address_line, self.alu.output),
                }
            },
            ORA(address_mode) => {
                self.load_alu_input_b(address_mode);
                self.do_accumulator_arithmetic(alu::Op::Or);
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
                self.load_alu_input_a(address_mode);
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Rol);
                match address_mode {
                    Accumulator => self.registers.A = value,
                    _ => self.memory.write_byte(self.address_line, value),
                }
                self.registers.P = flags;
            },
            ROR(address_mode) => {
                self.load_alu_input_a(address_mode);
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Ror);
                match address_mode {
                    Accumulator => self.registers.A = value,
                    _ => self.memory.write_byte(self.address_line, value),
                }
                self.registers.P = flags;
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
                self.load_alu_input_b(address_mode);
                self.do_accumulator_arithmetic(alu::Op::Sbc);
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
        self.memory.write_byte(self.registers.S as u16, value);
        self.alu.input_a = self.registers.S;
        self.alu.do_arithmetic(alu::Op::Dec);
        self.registers.S = self.alu.output;
    }

    fn push16(&mut self, value: u16) {
        self.memory.write_byte(self.registers.S as u16, (value >> 8).try_into().unwrap());
        self.alu.input_a = self.registers.S;
        self.alu.do_arithmetic(alu::Op::Dec);
        self.registers.S = self.alu.output;
        self.memory.write_byte(self.registers.S as u16, (value & 0xff).try_into().unwrap());
        self.alu.input_a = self.registers.S;
        self.alu.do_arithmetic(alu::Op::Dec);
        self.registers.S = self.alu.output;
    }

    #[inline]
    fn pull(&mut self) -> u8 {
        self.alu.input_a = self.registers.S;
        self.alu.do_arithmetic(alu::Op::Inc);
        self.registers.S = self.alu.output;
        self.memory.read_byte(self.registers.S as u16)
    }

    fn pull16(&mut self) -> u16 {
        self.alu.input_a = self.registers.S;
        self.alu.do_arithmetic(alu::Op::Inc);
        self.registers.S = self.alu.output;
        let low = self.memory.read_byte(self.registers.S as u16);
        self.alu.input_a = self.registers.S;
        self.alu.do_arithmetic(alu::Op::Inc);
        self.registers.S = self.alu.output;
        let high = self.memory.read_byte(self.registers.S as u16);
        ((high as u16) << 8) | (low as u16)
    }

    #[inline]
    fn store(&mut self, address_mode: AddressMode, value: u8) {

        match address_mode {
            Accumulator => self.registers.A = value,
            Immediate   => panic!("Attempt to store to an immediate operand!"),
            _ => {        
                self.calculate_address(address_mode);
                self.memory.write_byte(self.address_line, value);
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
                self.memory.read_byte(self.address_line)
            },
        };
        if value == 0 {
            self.registers.P.insert(Flags::Z);
        }
        if value & 0x80 == 0x80 {
            self.registers.P.insert(Flags::N);
        }
        value
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
                self.address_line = self.next_opcode.wrapping_add(self.registers.X) as u16;
            },
            ZeroPageY => {
                // Take the zero page address given by the operand and add the value of the Y register to it
                // This result of this calculation is wrapping, so that if the value is greater than 0xff then
                // any bits in the most significant byte will be zeroised so that the address remains a zero page one
                self.address_line = self.next_opcode.wrapping_add(self.registers.Y) as u16;
            },
            Relative => {
                // TODO: This is wrong. What actually happens is that the operand is a signed 8-bit integer
                // Need to check if representation is the same as within rust's primitive types
                // If so, we can transmute to i8 and do the addition
                self.alu.carry_in = false;
                self.alu.adc(self.next_opcode, (self.PC & 0xff) as u8);
                let low = self.alu.output;
                self.alu.carry_in = self.alu.carry_out;
                // Sign extend the 8 bit signed offset
                let a = match self.next_opcode & 0x80 {
                    0x80 => 0xff,
                    _    => 0x00
                };
                self.alu.adc(a, (self.PC >> 8) as u8);
                let high = self.alu.output;
                self.address_line = ((high as u16) << 8) | (low as u16);
            },
            Absolute => {
                // Operand is a 16-bit instruction

                // Little-endian read
                let low = self.next_opcode; let high = self.next();
                self.address_line = ((high as u16) << 8) | (low as u16);
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
                let load_location = ((high as u16) << 8) | (low as u16);
                self.address_line = load_location.saturating_add(self.registers.X as u16);
            },
            AbsoluteY => {
                // Exactly like (Absolute,X) addressing, but uses the Y register as the index
                let low = self.next_opcode; let high = self.next();
                let load_location = ((high as u16) << 8) | (low as u16);
                self.address_line =  load_location.saturating_add(self.registers.Y as u16);
            },
            Indirect => {
                // The immediate 16-bit operand points to the memory location of the effective address!
                let low = self.next_opcode; let high = self.next();
                let indirect = ((high as u16) << 8) | (low as u16);
                let low = self.memory.read_byte(indirect);
                let high = self.memory.read_byte(indirect.wrapping_add(1));
                self.address_line = ((high as u16) << 8) | (low as u16);
            },
            XIndirect => {
                // Take an 8-bit operand and add to the X-register, discarding the carry
                // This is a pointer to the zero page
                // It points to the LSB of the effective address, and the next location in the zero page points to 
                // the MSB of the effective address
                let indirect = self.next_opcode;
                let pointer = indirect.wrapping_add(self.registers.X) as u16;
                let low = self.memory.read_byte(pointer);
                let high = self.memory.read_byte(pointer.wrapping_add(1));
                self.address_line = ((high as u16) << 8) | (low as u16);
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
                let low = self.memory.read_byte(pointer);
                let high = self.memory.read_byte(pointer.wrapping_add(1));
                self.address_line = (((high as u16) << 8) | (low as u16)).wrapping_add(self.registers.Y as u16);
            },
        }
    }
}
