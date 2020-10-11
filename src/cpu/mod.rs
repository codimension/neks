use std::convert::TryInto;

mod instructions;
mod register;
mod alu;

use crate::ines::Cartridge;
use crate::memory::MemoryBus;

use register::{Flags, Registers};
use instructions::{AddressMode, Instruction};
use instructions::AddressMode::*;
use instructions::Instruction::*;
use alu::ALU;

const STACK_BEGIN: u8 = 0b100; // One above 0xff (empty stack)

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
}

impl CPU {
    pub fn init(cartridge: Cartridge) -> Self {
        Self {
            registers: Registers::init(),
            PC: 0,

            clock: 0,
            memory: MemoryBus::init(),
            cartridge: cartridge,
            alu: ALU::new(),

            address_line: 0,
            is_running: false,
        }
    }

    pub fn run(&mut self) -> () {
        self.is_running = true;
        while self.is_running {
            let opcode = self.next();
            let instruction = instructions::decode(opcode);
            self.execute(instruction);
        }
    }

    #[inline]
    pub fn tick(&mut self) {
        self.clock += 1;
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
                self.alu.input_b = self.next();
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
                self.alu.input_a = self.next();
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
            },
            DEX => {
                self.alu.input_a = self.registers.X;
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Dec);
                self.registers.P = flags;
                self.registers.X = value;
            },
            DEY => {
                self.alu.input_a = self.registers.Y;
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Dec);
                self.registers.P = flags;
                self.registers.Y = value;
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
            },
            INX => {
                self.alu.input_a = self.registers.X;
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Inc);
                self.registers.P = flags;
                self.registers.X = value;
            },
            INY => {
                self.alu.input_a = self.registers.Y;
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Inc);
                self.registers.P = flags;
                self.registers.Y = value;
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
                self.memory.write_byte(self.address_line, value);
                self.registers.P = flags;
            },
            ROR(address_mode) => {
                self.load_alu_input_a(address_mode);
                let (value, flags) = self.alu.do_arithmetic(alu::Op::Ror);
                self.memory.write_byte(self.address_line, value);
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
            },
            TAY => {
                self.registers.Y = self.registers.A;
            },
            TSX => {
                self.registers.X = self.registers.S;
            },
            TXA => {
                self.registers.A = self.registers.X;
            },
            TXS => {
                self.registers.S = self.registers.X;
            },
            TYA => {
                self.registers.A = self.registers.Y;
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
        self.registers.S -= 1;
    }

    fn push16(&mut self, value: u16) {
        self.memory.write_byte(self.registers.S as u16, (value >> 8).try_into().unwrap());
        self.registers.S -= 1;
        self.memory.write_byte(self.registers.S as u16, (value & 0xff).try_into().unwrap());
        self.registers.S -= 1;
    }

    #[inline]
    fn pull(&mut self) -> u8 {
        self.registers.S += 1;
        self.memory.read_byte(self.registers.S as u16)
    }

    fn pull16(&mut self) -> u16 {
        self.registers.S += 1;
        let low = self.memory.read_byte(self.registers.S as u16);
        self.registers.S += 1;
        let high = self.memory.read_byte(self.registers.S as u16);
        ((high as u16) << 8) | (low as u16)
    }

    #[inline]
    fn store(&mut self, address_mode: AddressMode, value: u8) {
        self.calculate_address(address_mode);
        self.memory.write_byte(self.address_line, value);
    }

    #[inline]
    fn load(&mut self, address_mode: AddressMode) -> u8 {
        self.calculate_address(address_mode);
        self.memory.read_byte(self.address_line)
    }

    #[inline]
    fn calculate_address(&mut self, address_mode: AddressMode) {
        match address_mode {
            Immediate => {
                // The value to use is the operand, no memory load is required
                panic!("Internal processor error: tried to do a load with Immediate value")
            },
            Implied => {
                // The value to use is taken from a register, depends on instruction
                panic!("Internal processor error: tried to do a load with Implied value")
            },
            ZeroPage => {
                // The operand for a Zero Page instruction gives the LSB of the memory location to load
                // The MSB is zero, so that the memory location loaded is in the 'zero page'
                self.address_line = self.next() as u16;
            },
            ZeroPageX => {
                // Take the zero page address given by the operand and add the value of the X register to it
                // This result of this calculation is wrapping, so that if the value is greater than 0xff then
                // any bits in the most significant byte will be zeroised so that the address remains a zero page one
                self.address_line = self.next().wrapping_add(self.registers.X) as u16;
            },
            ZeroPageY => {
                // Take the zero page address given by the operand and add the value of the Y register to it
                // This result of this calculation is wrapping, so that if the value is greater than 0xff then
                // any bits in the most significant byte will be zeroised so that the address remains a zero page one
                self.address_line = self.next().wrapping_add(self.registers.Y) as u16;
            },
            Relative => {
                // TODO: This is wrong. What actually happens is that the operand is a signed 8-bit integer
                // Need to check if representation is the same as within rust's primitive types
                // If so, we can transmute to i8 and do the addition
                self.address_line = (self.next() as u16) + self.PC;
            },
            Absolute => {
                // Operand is a 16-bit instruction

                // Little-endian read
                let low = self.next(); let high = self.next();
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
                let low = self.next(); let high = self.next();
                let load_location = ((high as u16) << 8) | (low as u16);
                self.address_line = load_location.saturating_add(self.registers.X as u16);
            },
            AbsoluteY => {
                // Exactly like (Absolute,X) addressing, but uses the Y register as the index
                let low = self.next(); let high = self.next();
                let load_location = ((high as u16) << 8) | (low as u16);
                self.address_line =  load_location.saturating_add(self.registers.Y as u16);
            },
            Indirect => {
                // The immediate 16-bit operand points to the memory location of the effective address!
                let low = self.next(); let high = self.next();
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
                let indirect = self.next();
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
                let pointer = self.next() as u16;
                let low = self.memory.read_byte(pointer);
                let high = self.memory.read_byte(pointer.wrapping_add(1));
                self.address_line = (((high as u16) << 8) | (low as u16)).wrapping_add(self.registers.Y as u16);
            },
        }
    }
}
