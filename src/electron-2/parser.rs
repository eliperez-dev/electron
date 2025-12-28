use super::{Instruction, Operation, OperationArgs, Operand, OperandType};
use std::collections::HashMap;

pub struct Parser;

impl Parser {
    pub fn parse(code: String) -> (Vec<Instruction>, Vec<String>, Vec<String>) {
        let lines: Vec<&str> = code.lines().collect();
        let mut instructions = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut labels = HashMap::new();
        let mut addr_counter = 0;

        // Pass 0: Scan labels
        for line in &lines {
            let clean = line.split(';').next().unwrap_or("").trim().to_uppercase();
            if let Some(idx) = clean.find(':') {
                if let Some(label) = clean.get(0..idx) {
                    if !label.contains(' ') {
                        labels.insert(label.to_string(), addr_counter);
                    }
                }
                let after = clean.get(idx+1..).unwrap_or("").trim();
                if !after.is_empty() {
                    addr_counter += 1;
                }
            } else if !clean.is_empty() {
                addr_counter += 1;
            }
        }

        // Pass 1: Parse
        addr_counter = 0;
        for (i, line) in lines.iter().enumerate() {
            let source_line = (i + 1) as i32;
            match Self::parse_line(line, addr_counter, source_line, &labels) {
                Ok(Some(instr)) => {
                    // 1. Static Warnings
                    let mut warns = Self::check_warnings(&instr, source_line);
                    
                    // 2. DYNAMIC HAZARD CHECK (Read-After-Write)
                    if let Some(prev) = instructions.last() {
                         // Check if previous instruction writes to a register
                        if let Some(written_reg) = Self::get_write_register(prev) {
                            // Check if current instruction reads that same register
                            let read_regs = Self::get_read_registers(&instr);
                            if read_regs.contains(&written_reg) {
                                warns.push(format!(
                                    "Line {}: RAW Hazard. Reading R{} immediately after writing may yield old value due to pipeline latency. Insert a NOOP.", 
                                    source_line, written_reg
                                ));
                            }
                        }
                    }

                    if !warns.is_empty() {
                        warnings.extend(warns);
                    }
                    instructions.push(instr);
                    addr_counter += 1;
                },
                Ok(None) => {}, // Empty or comment or just label
                Err(e) => {
                    errors.push(format!("Line {}: {}", source_line, e));
                }
            }
        }

        (instructions, errors, warnings)
    }

    fn check_warnings(instr: &Instruction, line: i32) -> Vec<String> {
        let mut warnings = Vec::new();
        let op = instr.operation;
        let a = &instr.a;
        let b = &instr.b;

        // 1. Check writing to R0
        let writes_to_a = matches!(op, 
            Operation::IMM | Operation::MOV | Operation::ADD | Operation::ADDC | 
            Operation::SUB | Operation::AND | Operation::OR | Operation::XOR | 
            Operation::SHR | Operation::NOT | Operation::LOAD | Operation::POP | 
            Operation::INP
        );

        if writes_to_a && a.type_ == OperandType::Register && a.data == 0 {
             let safe = match op {
                 Operation::ADD | Operation::ADDC | Operation::SUB | 
                 Operation::AND | Operation::OR | Operation::XOR => {
                     instr.args == OperationArgs::X
                 },
                 _ => false
             };

             if !safe {
                 warnings.push(format!("Line {}: Writing to Register 0 (Zero Register) effectively does nothing.", line));
             }
        }
        
        // 2. Out of bounds Immediate
        if a.type_ == OperandType::Immediate
            && (a.data < 0 || a.data > 255)
                 && !matches!(op, Operation::JMP | Operation::CALL | Operation::BIE | Operation::BIG | Operation::BIL | Operation::BIO) {
                     warnings.push(format!("Line {}: Immediate value {} is out of 8-bit range (0-255). It will be wrapped.", line, a.data));
                 }
        if b.type_ == OperandType::Immediate
            && (b.data < 0 || b.data > 255) {
                 warnings.push(format!("Line {}: Immediate value {} is out of 8-bit range (0-255). It will be wrapped.", line, b.data));
            }

        // 3. Port out of bounds
        if op == Operation::OUT
             && a.type_ == OperandType::Port
                 && (a.data < 0 || a.data > 7) {
                     warnings.push(format!("Line {}: Port %{} is out of range (0-7).", line, a.data));
                 }

        // 4. RAM out of bounds
        if op == Operation::STORE
             && a.type_ == OperandType::MemoryAddress
                 && (a.data < 0 || a.data > 15) {
                     warnings.push(format!("Line {}: Memory address #{} is out of RAM range (0-15).", line, a.data));
                 }
        if op == Operation::LOAD
             && b.type_ == OperandType::MemoryAddress
                 && (b.data < 0 || b.data > 15) {
                     warnings.push(format!("Line {}: Memory address #{} is out of RAM range (0-15).", line, b.data));
                 }

        warnings
    }

    // --- Helper Logic for Hazard Detection ---

    /// Returns the register index if the instruction writes to a register.
    fn get_write_register(instr: &Instruction) -> Option<i32> {
        // Must target a register
        if instr.a.type_ != OperandType::Register {
            return None;
        }

        match instr.operation {
            Operation::IMM | Operation::MOV | Operation::LOAD | Operation::POP | Operation::INP => Some(instr.a.data),
            Operation::ADD | Operation::ADDC | Operation::SUB | Operation::AND | Operation::OR | Operation::XOR => {
                // 'X' prefix writes to ACC only, not the Register
                if instr.args == OperationArgs::X {
                    None
                } else {
                    Some(instr.a.data)
                }
            },
            Operation::SHR | Operation::NOT => Some(instr.a.data),
            _ => None
        }
    }

    /// Returns a list of registers that are read by the instruction.
    fn get_read_registers(instr: &Instruction) -> Vec<i32> {
        let mut reads = Vec::new();

        // Check Operand A (Source)
        if instr.a.type_ == OperandType::Register {
            match instr.operation {
                // Math ops read A unless using U/X (which use ACC as source A)
                Operation::ADD | Operation::ADDC | Operation::SUB | Operation::AND | Operation::OR | Operation::XOR => {
                    if instr.args != OperationArgs::U && instr.args != OperationArgs::X {
                        reads.push(instr.a.data);
                    }
                },
                Operation::PUSH | Operation::ROUT => {
                    reads.push(instr.a.data);
                },
                _ => {}
            }
        }

        // Check Operand B (Source)
        if instr.b.type_ == OperandType::Register {
            match instr.operation {
                Operation::MOV | Operation::ADD | Operation::ADDC | Operation::SUB | 
                Operation::AND | Operation::OR | Operation::XOR | 
                Operation::SHR | Operation::NOT | Operation::OUT | 
                Operation::ROUT | Operation::STORE => {
                    reads.push(instr.b.data);
                },
                _ => {}
            }
        }

        reads
    }

    fn parse_line(line: &str, address: i32, source_line: i32, labels: &HashMap<String, i32>) -> Result<Option<Instruction>, String> {
        let mut clean = line.split(';').next().unwrap_or("").trim().to_uppercase();
        
        if let Some(idx) = clean.find(':') {
            clean = clean.get(idx+1..).unwrap_or("").trim().to_string();
        }

        if clean.is_empty() { return Ok(None); }

        let tokens: Vec<&str> = clean.split_whitespace().collect();
        if tokens.is_empty() { return Ok(None); }

        let (op, args) = Self::parse_operation(tokens[0])?;
        let needed = Self::get_needed_operands(op, args);

        let mut token_idx = 1;
        let mut val_a = Operand::new(OperandType::Immediate, 0);
        let mut val_b = Operand::new(OperandType::Immediate, 0);

        if needed.0
            && token_idx < tokens.len() {
                val_a = Self::parse_operand(tokens[token_idx], labels)?;
                token_idx += 1;
            }
        if needed.1
            && token_idx < tokens.len() {
                val_b = Self::parse_operand(tokens[token_idx], labels)?;
                token_idx += 1;
            }

        Ok(Some(Instruction {
            operation: op,
            args,
            a: val_a,
            b: val_b,
            address,
            source_line,
        }))
    }

    fn parse_operation(s: &str) -> Result<(Operation, OperationArgs), String> {
        if let Some(op) = Self::match_op(s) {
            return Ok((op, OperationArgs::None));
        }
        
        // Check prefixes
        let prefix = s.chars().next().unwrap();
        let suffix = &s[1..];
        if let Some(op) = Self::match_op(suffix) {
            let args = match prefix {
                'S' => OperationArgs::S,
                'U' => OperationArgs::U,
                'X' => OperationArgs::X,
                _ => return Err(format!("Invalid operation: {}", s)),
            };
            return Ok((op, args));
        }

        Err(format!("Invalid operation: {}", s))
    }

    fn match_op(s: &str) -> Option<Operation> {
        match s {
            "NOOP" | "NOP" => Some(Operation::NOOP),
            "IMM" => Some(Operation::IMM),
            "MOV" => Some(Operation::MOV),
            "ADD" => Some(Operation::ADD),
            "ADDC" => Some(Operation::ADDC),
            "SUB" => Some(Operation::SUB),
            "OR" => Some(Operation::OR),
            "XOR" => Some(Operation::XOR),
            "AND" => Some(Operation::AND),
            "SHR" => Some(Operation::SHR),
            "NOT" => Some(Operation::NOT),
            "OUT" => Some(Operation::OUT),
            "ROUT" => Some(Operation::ROUT),
            "INP" => Some(Operation::INP),
            "JMP" => Some(Operation::JMP),
            "BIE" => Some(Operation::BIE),
            "BIG" => Some(Operation::BIG),
            "BIL" => Some(Operation::BIL),
            "BIO" => Some(Operation::BIO),
            "STORE" => Some(Operation::STORE),
            "LOAD" => Some(Operation::LOAD),
            "PUSH" => Some(Operation::PUSH),
            "POP" => Some(Operation::POP),
            "CALL" => Some(Operation::CALL),
            "RET" => Some(Operation::RET),
            _ => None
        }
    }

    fn get_needed_operands(op: Operation, args: OperationArgs) -> (bool, bool) {
        match op {
            Operation::NOOP | Operation::RET => (false, false),
            Operation::IMM | Operation::MOV | Operation::SHR | Operation::NOT | 
            Operation::OUT | Operation::STORE | Operation::LOAD | Operation::ROUT => (true, true),
            
            Operation::ADD | Operation::ADDC | Operation::SUB | 
            Operation::OR | Operation::XOR | Operation::AND => {
                if args == OperationArgs::X { (false, true) } else { (true, true) }
            },

            Operation::JMP | Operation::BIE | Operation::BIG | 
            Operation::BIL | Operation::BIO | Operation::INP | 
            Operation::PUSH | Operation::POP | Operation::CALL => (true, false),
        }
    }

    fn parse_operand(s: &str, labels: &HashMap<String, i32>) -> Result<Operand, String> {
        let first = s.chars().next().ok_or("Empty operand")?;
        let rest = &s[1..];

        if first == 'R' || first == '$' {
            if let Ok(val) = Self::parse_binary(rest) {
                return Ok(Operand::new(OperandType::Register, val));
            }
        } 
        
        if first == '#' || first == '@' {
            let val = Self::parse_binary(rest)?;
            return Ok(Operand::new(OperandType::MemoryAddress, val));
        }
        
        if first == '%' {
            let val = Self::parse_binary(rest)?;
            return Ok(Operand::new(OperandType::Port, val));
        }

        // Immediate or Label
        if let Ok(val) = Self::parse_binary(s) {
             Ok(Operand::new(OperandType::Immediate, val))
        } else {
                // Label lookup
                if let Some(&addr) = labels.get(s) {
                    Ok(Operand::new(OperandType::Immediate, addr))
                } 
                else {
                      Err(format!("Invalid value or unknown label: {}", s))
            }
        }
    }

    fn parse_binary(s: &str) -> Result<i32, String> {
        let clean = s.replace('_', "");
        if clean.starts_with('B') {
            i32::from_str_radix(&clean[1..], 2).map_err(|_| format!("Invalid binary: {}", s))
        } else {
            clean.parse::<i32>().map_err(|_| format!("Invalid number: {}", s))
        }
    }
}