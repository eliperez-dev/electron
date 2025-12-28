pub mod parser;
use parser::Parser;

// --- Enums & Types ---

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operation {
    NOOP, IMM, MOV, ADD, ADDC, SUB, OR, XOR, AND, SHR, NOT,
    OUT, ROUT, INP, JMP, BIE, BIG, BIL, BIO, STORE, LOAD,
    PUSH, POP, CALL, RET
}

impl Operation {
    pub fn get_name(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OperationArgs {
    None, S, U, X
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OperandType {
    Register = 0,
    MemoryAddress = 1,
    Immediate = 2,
    Port = 3
}

#[derive(Clone, Copy, Debug)]
pub struct Operand {
    pub type_: OperandType,
    pub data: i32, 
}

impl Operand {
    pub fn new(type_: OperandType, data: i32) -> Self {
        Self { type_, data }
    }
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub operation: Operation,
    pub args: OperationArgs,
    pub a: Operand,
    pub b: Operand,
    pub address: i32,
    pub source_line: i32,
}

impl Instruction {
    pub fn none() -> Self {
        Self {
            operation: Operation::NOOP,
            args: OperationArgs::None,
            a: Operand::new(OperandType::Immediate, 0),
            b: Operand::new(OperandType::Immediate, 0),
            address: -1,
            source_line: 0,
        }
    }
}

// --- Components ---

pub struct Registers {
    pub regs: [u8; 8],
    pub next_regs: [u8; 8],
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

impl Registers {
    pub fn new() -> Self {
        Self {
            regs: [0; 8],
            next_regs: [0; 8],
        }
    }

    pub fn begin_cycle(&mut self) {
        self.next_regs = self.regs;
    }

    pub fn end_cycle(&mut self) {
        self.regs = self.next_regs;
    }

    pub fn read(&self, addr: i32) -> u8 {
        if addr <= 0 || addr > 7 {
            0
        } else {
            self.regs[addr as usize]
        }
    }

    pub fn write(&mut self, addr: i32, data: u8) {
        if addr > 0 && addr < 8 {
            self.next_regs[addr as usize] = data;
        }
    }
    
    pub fn get_all(&self) -> Vec<u8> {
        self.regs.to_vec()
    }
}

pub struct AluFlags {
    pub equals: bool,
    pub greater: bool,
    pub less: bool,
    pub overflow: bool,
}

pub struct ALU {
    pub accumulator: u8,
    pub flags: AluFlags,
}

impl Default for ALU {
    fn default() -> Self {
        Self::new()
    }
}

impl ALU {
    pub fn new() -> Self {
        Self {
            accumulator: 0,
            flags: AluFlags { equals: false, greater: false, less: false, overflow: false },
        }
    }

    pub fn execute(&mut self, registers: &Registers, instr: &Instruction, input_register: &mut i32, waiting_for_input: &mut bool) {
        let a_data = if instr.args == OperationArgs::U || instr.args == OperationArgs::X {
            self.accumulator
        } else {
            registers.read(instr.a.data)
        };

        let b_data = registers.read(instr.b.data);

        
        let mut result: i32 = 0;
        let op = instr.operation;

        match op {
            Operation::ADD => result = (a_data as i32) + (b_data as i32),
            Operation::ADDC => {
                let carry = if self.flags.overflow { 1 } else { 0 };
                result = (a_data as i32) + (b_data as i32) + carry;
            },
            Operation::SUB => result = (a_data as i32) - (b_data as i32),
            Operation::OR => result = (a_data as i32) | (b_data as i32),
            Operation::XOR => result = (a_data as i32) ^ (b_data as i32),
            Operation::AND => result = (a_data as i32) & (b_data as i32),
            Operation::SHR => result = (b_data as i32) >> 1,
            Operation::NOT => result = (!b_data as i32) & 0xFF,
            Operation::INP => {
                *waiting_for_input = true;
                *input_register = instr.a.data;
                result = 0;
            },
            _ => {}
        }

        // Store Accumulator
        let is_alu_op = matches!(op, 
            Operation::ADD | Operation::ADDC | Operation::SUB | 
            Operation::OR | Operation::XOR | Operation::AND | 
            Operation::SHR | Operation::NOT
        );

        if is_alu_op {
            // Flags
            self.flags.equals = a_data == b_data;
            self.flags.greater = a_data > b_data;
            self.flags.less = a_data < b_data;
            self.flags.overflow = !(0..=255).contains(&result);
            
            self.accumulator = (result & 0xFF) as u8;
        }
    }
}

// --- Emulator ---

pub struct Emulator {
    pub instructions: Vec<Instruction>,
    pub pc: i32,
    pub sp: i32,
    
    pub fetch_reg: Instruction,
    pub decode_reg: Instruction,
    pub execute_reg: Instruction,
    pub writeback_reg: Instruction,

    pub registers: Registers,
    pub alu: ALU,
    pub ports_out: [u8; 8],
    pub ram: [u8; 16],

    pub waiting_for_input: bool,
    pub input_register: i32,

    // Diagnostics
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Emulator {
    pub fn new(code: String) -> Emulator {
        let mut emu = Emulator {
            instructions: Vec::new(),
            pc: 0,
            sp: 15,
            fetch_reg: Instruction::none(),
            decode_reg: Instruction::none(),
            execute_reg: Instruction::none(),
            writeback_reg: Instruction::none(),
            registers: Registers::new(),
            alu: ALU::new(),
            ports_out: [0; 8],
            ram: [0; 16],
            waiting_for_input: false,
            input_register: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        emu.load_program(code);
        emu
    }

    pub fn load_program(&mut self, code: String) {
        self.instructions.clear();
        self.errors.clear();
        self.warnings.clear();
        self.pc = 0;
        self.sp = 15;
        self.reset_state();

        let (instrs, errs, warns) = Parser::parse(code);
        self.instructions = instrs;
        self.errors = errs;
        self.warnings = warns;
    }
    
    fn reset_state(&mut self) {
        self.registers = Registers::new();
        self.alu = ALU::new();
        self.fetch_reg = Instruction::none();
        self.decode_reg = Instruction::none();
        self.execute_reg = Instruction::none();
        self.writeback_reg = Instruction::none();
        self.ports_out = [0; 8];
        self.ram = [0; 16];
        self.waiting_for_input = false;
    }

    pub fn clock(&mut self) {
        if self.waiting_for_input { return; }

        self.registers.begin_cycle();

        // Pipeline (Reverse)
        self.write_back_stage();
        self.execute_stage();
        self.decode_stage();
        self.fetch_stage();

        self.increment_pc();
        self.registers.end_cycle();
    }

    pub fn resolve_input(&mut self, val: i32) {
        if self.waiting_for_input {
            self.alu.accumulator = (val & 0xFF) as u8;
            self.waiting_for_input = false;
        }
    }
    
    // --- Internal Pipeline ---
    fn increment_pc(&mut self) {
        self.pc += 1;
        if self.pc >= 255 { self.pc = 0; }
    }

    fn fetch_stage(&mut self) {
        if self.pc >= 0 && (self.pc as usize) < self.instructions.len() {
            self.fetch_reg = self.instructions[self.pc as usize].clone();
        } else {
            self.fetch_reg = Instruction::none();
        }
    }

    fn decode_stage(&mut self) {
        self.decode_reg = self.fetch_reg.clone();
    }

    fn execute_stage(&mut self) {
        self.execute_reg = self.decode_reg.clone();
        let op = self.execute_reg.operation;

        // Branching
        let mut take_branch = false;
        if op == Operation::JMP { take_branch = true; }
        else if op == Operation::CALL { take_branch = true; }
        else if op == Operation::BIE && self.alu.flags.equals { take_branch = true; }
        else if op == Operation::BIG && self.alu.flags.greater { take_branch = true; }
        else if op == Operation::BIO && self.alu.flags.overflow { take_branch = true; }
        else if op == Operation::BIL && self.alu.flags.less { take_branch = true; }
        else if op == Operation::RET {
            take_branch = true;
            self.sp += 1;
            if self.sp > 15 { self.sp = 0; }
            let ret_addr = self.ram[self.sp as usize];
            self.execute_reg.a.data = ret_addr as i32; // Hack to use common branch logic
        }

        if take_branch {
            self.pc = self.execute_reg.a.data;
            self.fetch_reg = Instruction::none(); // Flush
        }

        self.alu.execute(&self.registers, &self.execute_reg, &mut self.input_register, &mut self.waiting_for_input);
    }

    fn write_back_stage(&mut self) {
        self.writeback_reg = self.execute_reg.clone();
        let op = self.writeback_reg.operation;
        let a = self.writeback_reg.a.data;
        let b = self.writeback_reg.b.data;
        let address = self.writeback_reg.address;

        match op {
            Operation::IMM => self.registers.write(a, b as u8),
            Operation::MOV => {
                let val = self.registers.read(b);
                self.registers.write(a, val);
            },
            Operation::ADD | Operation::ADDC | Operation::SUB | 
            Operation::OR | Operation::XOR | Operation::AND => {
                let args = self.writeback_reg.args;
                if args == OperationArgs::S || args == OperationArgs::U || args == OperationArgs::None {
                    self.registers.write(a, self.alu.accumulator);
                }
            },
            Operation::SHR | Operation::NOT => {
                self.registers.write(a, self.alu.accumulator);
            },
            Operation::INP => {
                self.registers.write(a, self.alu.accumulator);
            },
            Operation::OUT => {
                if a < 8 {
                    self.ports_out[a as usize] = self.registers.read(b);
                }
            },
            Operation::ROUT => {
                if self.registers.read(a) < 8 {
                    self.ports_out[self.registers.read(a) as usize] = self.registers.read(b);
                }
            },
            Operation::STORE => {
                if a < 16 {
                    self.ram[a as usize] = self.registers.read(b);
                }
            },
            Operation::LOAD => {
                if b < 16 {
                    self.registers.write(a, self.ram[b as usize]);
                }
            },
            Operation::PUSH => {
                if self.sp >= 0 {
                    self.ram[self.sp as usize] = self.registers.read(a);
                    self.sp -= 1;
                    if self.sp < 0 { self.sp = 15; }
                }
            },
            Operation::POP => {
                self.sp += 1;
                if self.sp > 15 { self.sp = 0; }
                self.registers.write(a, self.ram[self.sp as usize]);
            },
            Operation::CALL => {
                if self.sp >= 0 {
                    self.ram[self.sp as usize] = (address + 1) as u8;
                    self.sp -= 1;
                    if self.sp < 0 { self.sp = 15; }
                }
            },
            _ => {}
        }
    }
}