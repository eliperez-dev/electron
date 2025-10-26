# Electron CPU Architecture & ISA

This document provides a comprehensive guide to the Electron 8-bit CPU architecture, including hardware specifications, the complete instruction set, the pipeline model, and programming guidelines.

---

## Table of Contents

1. [Hardware Specifications](#hardware-specifications)
2. [The 4-Stage Waterfall Pipeline](#the-4-stage-waterfall-pipeline)
3. [Instruction Set Architecture (ISA)](#instruction-set-architecture-isa)
4. [Operand Types](#operand-types)
5. [ALU & Flags](#alu--flags)
6. [Memory & Registers](#memory--registers)
7. [I/O System](#io-system)
8. [Programming Guide](#programming-guide)
9. [Pipeline Timing & Latency](#pipeline-timing--latency)

---

## Hardware Specifications

### Core Parameters

| Component | Specification |
|-----------|---|
| **Architecture** | 8-bit, single-core, RISC-inspired |
| **Word Size** | 1 byte (8 bits) |
| **Value Range** | 0–255 (unsigned) |
| **Pipeline Depth** | 4 stages (waterfall) |
| **Bus Width** | 8 bits |

### Memory & Storage

| Memory Type | Size | Purpose |
|---|---|---|
| **ROM (Instruction Memory)** | 96 bytes | Holds up to 32 instructions (3 bytes each) |
| **Registers (General Purpose)** | 8 registers (8 bits each) | R0–R7; R0 is read-only zero |
| **Accumulator** | 1 register (8 bits) | ALU result storage |

### I/O Interface

| Resource | Specification |
|---|---|
| **Output Ports** | 8 ports (8 bits each) |
| **Input Ports** | 1 port (8 bits) |
| **Display** | 8×8 LED matrix (64 pixels) |
| **Total Output Data** | 64 bits = 8 bytes |

### Instruction Format

- **Instruction Size:** 3 bytes per instruction
- **Maximum Program Length:** 32 instructions (96 bytes)
- **Program Counter Width:** 5 bits (addresses 0–31)
- **Auto-Wrap:** When PC exceeds 31, it wraps to 0

---

## The 4-Stage Waterfall Pipeline

The Electron uses a **synchronous, non-overlapping 4-stage waterfall pipeline**. Each instruction progresses through exactly four stages, one per clock cycle.

### Pipeline Stages

1. **FETCH** — Retrieve instruction from ROM at address PC
2. **DECODE** — Pass instruction through the decode register (minimal decoding)
3. **EXECUTE** — Perform ALU operations, check for branch conditions, update flags
4. **WRITE_BACK** — Write results to registers, ports, or memory

### Visual Pipeline Example

```
Cycle→  0     1      2       3        4       5
       ─────────────────────────────────────────
PC=0   [F]
PC=1       [F]  [D]
PC=2           [F]  [D]   [E]
PC=3                [F]  [D]   [E]  [WB]
PC=4                     [F]  [D]   [E]  [WB]
PC=5                          [F]  [D]   [E]  [WB]
       
Legend: F=Fetch, D=Decode, E=Execute, WB=Write Back
```

### Key Pipeline Behaviors

**Instruction Latency:**
- An instruction takes **4 clock cycles minimum** to complete (F→D→E→WB)
- Results are **not available** until the WRITE_BACK stage
- This means the result of an ALU operation cannot be used by the next instruction

**Branch Handling:**
- Branch operations (JMP, BIE) take effect **immediately in the EXECUTE stage**
- However, the next 3 instructions already in the pipeline complete before the new PC address is fetched
- **Result:** Branches incur a 3-cycle penalty (3 instructions execute after branch before new address fetches)

**Pipeline Flushing:**
- When a branch is taken, the DECODE and FETCH stages continue with old PC values
- Instructions already fetched are still executed (no true pipeline flush)

### Implications for Programmers

- **No data hazards** — results aren't available immediately, so the next instruction cannot use the previous result
- **Predictable timing** — every instruction takes 4 cycles; branching takes additional cycles
- **Dead cycles after ALU** — often necessary to insert NOOP or other independent instructions after ALU operations
- **Write back is where it counts** — the WRITE_BACK stage determines where results go (registers, ports, or nowhere)

---

## Instruction Set Architecture (ISA)

The Electron supports **10 core instructions**, organized by category.

### 1. Data Movement Instructions

#### `IMM` — Immediate Load
```
Syntax:  IMM <register> <immediate>
Example: IMM R1 42
Format:  Operation=IMM, A=Register, B=Immediate
Operands: Reg (0-7), Imm (0-255)
Cycles:  4 (standard)

Behavior:
  - Load 8-bit immediate value into register A
  - Happens in WRITE_BACK stage
  - Does not modify ALU or flags
```

#### `MOV` — Register-to-Register Move
```
Syntax:  MOV <dest-register> <src-register>
Example: MOV R1 R2
Format:  Operation=MOV, A=Dest Reg, B=Src Reg
Operands: Reg (0-7), Reg (0-7)
Cycles:  4 (standard)

Behavior:
  - Copy value from register B to register A
  - Destination must be a register (not immediate)
  - Happens in WRITE_BACK stage
  - Does not modify ALU or flags
```

---

### 2. Arithmetic/Logic Instructions (ALU Operations)

All ALU operations use the Accumulator as the result register and set condition flags.

#### `ADD` — Addition
```
Syntax:  ADD <register-a> <register-b>
         SADD <register-a> <register-b>  (store to A)
         UADD <register-a> <register-b>  (use accumulator for A)
         XADD <register-a> <register-b>  (use accumulator, no store to A)

Examples:
  ADD R1 R2      - Compare/compute R1 + R2 (result in accumulator only)
  SADD R1 R2     - R1 + R2 → Accumulator → R1
  UADD R3 R4     - Accumulator + R4 → Accumulator → R3
  XADD R1 R2     - Accumulator + R2 → Accumulator (no write to R1)

Operands: Register, Register
Cycles:  4 (standard)

ALU Operation:
  operand_a = (modifier == U or X) ? accumulator : read(register_a)
  operand_b = read(register_b)
  accumulator = operand_a + operand_b

Flags Set:
  equals       = (operand_a == operand_b)
  greater_than = (operand_a > operand_b)
  less_than    = (operand_a < operand_b)
  overflow     = (accumulator > 255)

Result Storage (in WRITE_BACK):
  S variant: accumulator → register_a
  U variant: accumulator → register_a
  X variant: no write (flags only)
  None:      no write (flags only)
```

#### `ADDC` — Add with Carry
```
Syntax:  ADDC <register-a> <register-b>
         SADDC, UADDC, XADDC (variants as above)

Examples:
  ADDC R1 R2     - R1 + R2 + 1 → Accumulator
  SADDC R1 R2    - R1 + R2 + 1 → Accumulator → R1

Operands: Register, Register
Cycles:  4 (standard)

ALU Operation:
  operand_a = (modifier == U or X) ? accumulator : read(register_a)
  operand_b = read(register_b)
  accumulator = operand_a + operand_b + 1

Flags Set: Same as ADD

Result Storage: Same as ADD
```

#### `SHR` — Shift Right (Logical)
```
Syntax:  SHR <register-a> <register-b>

Example: SHR R1 R2     - R2 >> 1 → Accumulator → R1

Operands: Register, Register
Cycles:  4 (standard)

ALU Operation:
  accumulator = read(register_b) >> 1

Flags Set:
  equals       = (accumulator == read(register_b))
  greater_than = (accumulator > read(register_b))
  less_than    = (accumulator < read(register_b))
  overflow     = false (shift right never overflows)

Result Storage: accumulator → register_a (always writes)
```

#### `NOT` — Bitwise Complement
```
Syntax:  NOT <register-a> <register-b>

Example: NOT R1 R2     - ~R2 → Accumulator → R1

Operands: Register, Register
Cycles:  4 (standard)

ALU Operation:
  accumulator = !read(register_b)  (bitwise NOT / complement)

Flags Set: Same structure as SHR

Result Storage: accumulator → register_a (always writes)
```

---

### 3. I/O Instructions

#### `OUT` — Output to Port
```
Syntax:  OUT <port> <register>

Example: OUT %0 R1     - Write R1 to output port 0

Operands: Port (0-7), Register (0-7)
Cycles:  4 (standard)

Behavior:
  - Read value from register
  - Write to output port in WRITE_BACK stage
  - Ports are visualized on the 8×8 LED display
  - Does not affect ALU or registers

Port Mapping (8×8 LED Display):
  Port 0 → Display row 0, columns 0-7 (8 bits)
  Port 1 → Display row 1, columns 0-7
  Port 2 → Display row 2, columns 0-7
  ...
  Port 7 → Display row 7, columns 0-7
```

---

### 4. Control Flow Instructions

#### `JMP` — Unconditional Jump
```
Syntax:  JMP <address>

Example: JMP 5        - Jump to instruction 5

Operands: Immediate (0-31)
Cycles:  4 (standard, but with pipeline penalty)

Behavior:
  - In EXECUTE stage, update program counter to the specified address
  - Does not modify registers, ports, or flags
  - Next 3 instructions already in pipeline complete before fetch from new address

Pipeline Effect:
  Instruction N (JMP):     [F] → [D] → [E: PC updated] → [WB]
  Instructions N+1, N+2:   [E] → [WB]  (already in pipeline)
  Instruction N+3:         [D]
  Instruction <new>:       [F] (new address)

Result: 4-cycle penalty to redirect execution
```

#### `BIE` — Branch If Equal
```
Syntax:  BIE <address>

Example: BIE 10       - Jump to instruction 10 if equals flag is true

Operands: Immediate (0-31)
Cycles:  4 (standard, but with pipeline penalty if branch taken)

Behavior:
  - In EXECUTE stage, check the "equals" flag (set by previous ALU operation)
  - If equals flag is true: update program counter to address
  - If equals flag is false: program counter increments normally
  - Does not modify registers or flags (only reads flags)

Pipeline Effect: Same as JMP when branch is taken
```

---

### 5. No Operation

#### `NOOP` — No Operation
```
Syntax:  NOOP (or NOP)

Example: NOOP

Operands: None
Cycles:  4 (standard)

Behavior:
  - Execute all 4 pipeline stages without doing anything
  - Useful for:
    - Padding programs to 32 instructions
    - Creating pipeline delays between dependent operations
    - Flushing partial results
```

---

## Operand Types

When writing Electron assembly (`.elt` files), operands use specific prefixes and formats:

### Register Operand
```
Syntax:  R<n>
Range:   n ∈ [0, 7]
Example: R1, R7
Meaning: General-purpose register n

Special: R0 is always zero (read-only)
         Writing to R0 does nothing
         Reading R0 always returns 0
```

### Output Port Operand
```
Syntax:  %<n>
Range:   n ∈ [0, 7]
Example: %0, %3
Meaning: I/O output port n (used only with OUT instruction)
```

### Immediate (Literal) Operand
```
Syntax:  <decimal> or B<binary>
Range:   0–255 (8-bit unsigned)
Example: 42, 255, 0

Binary Format:
  B11001010      - Binary literal (8 bits) - PREFIX MUST BE 'B', NOT '0b'
  B1100_1010     - Binary with underscores for readability
```

### Assembly Operand Validation

The assembler enforces operand type matching:
- `IMM` requires: Register, Immediate
- `MOV` requires: Register, Register
- `ADD`/`ADDC` require: Register, Register
- `SHR`/`NOT` require: Register, Register
- `OUT` requires: Port, Register
- `JMP`/`BIE` require: Immediate
- `NOOP` requires: (no operands)

---

## ALU & Flags

### Arithmetic Logic Unit (ALU)

The ALU processes all arithmetic and bitwise operations:

**ALU Components:**
- **Accumulator:** 8-bit register holding ALU results
- **Flag Register:** 4-bit register with condition flags
- **Inputs:** Two 8-bit operands (from registers or accumulator)
- **Output:** 8-bit result + 4-bit flags

### Condition Flags

After every ALU operation, four flags are set based on the operation:

| Flag | Name | Set When |
|---|---|---|
| **EQ** | Equals | operand_a == operand_b |
| **GT** | Greater Than | operand_a > operand_b |
| **LT** | Less Than | operand_a < operand_b |
| **OV** | Overflow | result > 255 (8-bit overflow) |

### Flag Usage

- **BIE instruction** reads the Equals flag to conditionally branch
- Flags set by ALU operations persist until the next ALU operation
- Non-ALU instructions (IMM, MOV, OUT, etc.) do not modify flags
- This allows checking flags multiple operations later (though pipeline latency applies)

### Example: Using Flags for Conditional Logic

```assembly
IMM R1 100
IMM R2 100
ADD R1 R2        ; Sets equals=true, overflow=false
BIE 20           ; Jumps to instruction 20 because equals=true

; vs.

IMM R3 50
IMM R4 75
ADD R3 R4        ; Sets equals=false
BIE 20           ; Does NOT jump because equals=false
```

---

## Memory & Registers

### General Purpose Registers (8 total)

| Register | Name | Size | Special Use |
|---|---|---|---|
| R0 | Zero Register | 8 bits | Always reads as 0; writes are ignored |
| R1 | General Purpose | 8 bits | No special function |
| R2 | General Purpose | 8 bits | No special function |
| R3 | General Purpose | 8 bits | No special function |
| R4 | General Purpose | 8 bits | No special function |
| R5 | General Purpose | 8 bits | No special function |
| R6 | General Purpose | 8 bits | No special function |
| R7 | General Purpose | 8 bits | No special function |

### Accumulator

The **Accumulator** is a dedicated ALU result register:
- **Width:** 8 bits
- **Read:** Can be used as an operand in ADD/ADDC with U or X modifiers
- **Write:** ALU operations write results here (intermediate storage)
- **Final Storage:** WRITE_BACK stage copies accumulator to destination register

### Program Counter (PC)

- **Width:** 5 bits (addresses 0–31)
- **Initial Value:** 0 (starts at instruction 0)
- **Increment:** Automatically increments by 1 each cycle (after execution)
- **Wrap:** At 31 → 0 (continuous loop)
- **Modification:** JMP and BIE can set PC to a new address

---

## I/O System

### Output Ports

The CPU has **8 output ports**, each 8 bits wide, totaling 64 bits of output.

#### Port Layout (8×8 LED Display)

```
Port 0: [B7] [B6] [B5] [B4] [B3] [B2] [B1] [B0]  ← Display Row 0
        Col7 Col6 Col5 Col4 Col3 Col2 Col1 Col0

Port 1: Display Row 1
Port 2: Display Row 2
...
Port 7: Display Row 7
```

**Bit Mapping:**
- Bit value 1 = LED ON (white pixel)
- Bit value 0 = LED OFF (black pixel)

#### Example: Drawing a Pattern

```assembly
IMM R1 B11110000   ; 0xF0 - top half on
IMM R2 B00001111   ; 0x0F - bottom half on
OUT %0 R1          ; Write to row 0
OUT %1 R2          ; Write to row 1
```

This displays:
```
████░░░░   (Port 0)
░░░░████   (Port 1)
```

### Input Ports

- **Single input port** (currently unused in most programs)
- Can read external sensor data or user input
- 8 bits wide

---

## Programming Guide

### Assembly Language Syntax

```
; Comments start with semicolon
; Each line is one instruction

INSTRUCTION_NAME operand1 operand2  ; Optional inline comment
```

### Valid Examples

```assembly
; Data movement
IMM R1 42
MOV R2 R1

; Arithmetic
ADD R1 R2
SADD R3 R4      ; S variant: store result to R3
UADD R5 R2      ; U variant: use accumulator as first operand

; I/O
OUT %0 R1       ; Display R1 on port 0

; Control
JMP 0           ; Jump to instruction 0
BIE 10          ; Jump to instruction 10 if equals flag is set

; No-op
NOOP
```

### Assembler Features

- **Case-insensitive:** `IMM`, `imm`, `Imm` all valid
- **Binary literals:** `B11001010` or `B1100_1010` (uppercase `B` prefix only, not `0b`)
- **Inline comments:** Text after `;` ignored
- **Blank lines:** Skipped during parsing
- **Automatic padding:** Programs < 32 instructions padded with NOOP

### Assembler Validation

The assembler performs two checks:

1. **Error Checking:** Syntax errors, invalid operands, unknown instructions
   - Errors prevent the program from loading
   - Error messages show line number and issue

2. **Warning Checking:** Type mismatches (e.g., using a register where immediate expected)
   - Warnings are printed but don't prevent loading
   - Useful for catching logic errors

### Example Program Validation

```
0:  IMM R1 100
    [Output]: IMM, None, Register(1), Immediate(100)

1:  SADD R1 R2
    [Output]: ADD, S, Register(1), Register(2)
    [Warning]: ADD takes an immediate for operand B, not a register

Successfully validated program.
```

---

### Compiled Program Format

After successful assembly:
1. Instructions are stored in ROM (up to 32)
2. Programs shorter than 32 instructions are padded with NOOP
3. Each instruction occupies 1 ROM address
4. Program automatically loops when PC wraps from 31 to 0

---

### Running Programs

```bash
# Compile and run with emulator GUI
cargo run -- -f program.elt

# Optional flags
-c <speed>   # Clock speed multiplier (default: 1.0)
-nt          # No terminal output (GUI only)
-fps         # Show frames per second
```

---

## Pipeline Timing & Latency

### Instruction Latency Breakdown

**Standard instruction (4 cycles):**
```
Cycle 1: FETCH   [Retrieve from ROM]
Cycle 2: DECODE  [Move to decode register]
Cycle 3: EXECUTE [Perform operation / check branch]
Cycle 4: WRITE_BACK [Store results]
```

### Result Availability

**Critical:** Results are **not available until WRITE_BACK completes**.

```assembly
IMM R1 10       ; Cycle 0-3: Load 10 into R1
UADD R2 R1      ; Cycle 4-7: Use R1 (available in cycle 4)
                ; ^ This works because IMM completes in cycle 3
```

However:

```assembly
ADD R1 R2       ; Cycle 0-3: R1 + R2 → Accumulator
SADD R3 R4      ; Cycle 4-7: R3 + R4 → Accumulator (uses NEW value in R3?)
                ; ^ Problem: R3 not written until cycle 3's WRITE_BACK
                ; In cycle 4, R3 still holds old value!
```

### Pipeline Hazards & Solutions

**Data Hazard Example:**
```assembly
ADD R1 R2       ; Result goes to Accumulator, then R1 (cycle 3 WRITE_BACK)
UADD R1 R3      ; Tries to use R1 at cycle 4, but R1 updated at cycle 3
                ; Result: Uses STALE value of R1
```

**Solution 1: Use NOOP as spacer**
```assembly
ADD R1 R2
NOOP            ; Wait one cycle
UADD R1 R3      ; Now R1 is updated
```

**Solution 2: Use the Accumulator directly**
```assembly
ADD R1 R2       ; Result in Accumulator
UADD R3 R4      ; Use Accumulator (updated immediately after ADD's EXECUTE)
                ; ^ This is safe because accumulator updates in EXECUTE stage
```

### Branch Penalty

**Branches incur extra cycles:**
```
Instruction N (JMP):       [F] → [D] → [E] → [WB]
Instr N+1 (already fetched): [D] → [E] → [WB]
Instr N+2 (already fetched): [E] → [WB]
Instr N+3 (already fetched): [WB]
New target instruction:     [F] → [D] → [E] → [WB]
```

**Result:** 4 extra cycles to redirect execution (3 dead instructions + 1 to fetch new address)

---

### Performance Tips

1. **Minimize branching** — Each branch adds ~4 cycles of overhead
2. **Use NOOP strategically** — After ALU operations, insert NOOP if next instruction depends on result
3. **Unroll loops** — When possible, inline instructions instead of looping
4. **Batch operations** — Group independent ALU operations back-to-back
5. **Pipeline-aware programming** — Design code with 4-cycle stages in mind

---

## Example Programs

### 1. Simple Counter

```assembly
IMM R1 0        ; Initialize counter to 0
LOOP:
  OUT %0 R1     ; Display R1 on port 0
  IMM R2 1      ; Load 1
  UADD R1 R0    ; Increment R1 by 1
  JMP LOOP      ; Loop forever
```

**Behavior:** Counts 0–255 continuously on port 0 (8x8 display row 0).

### 2. Fibonacci Sequence

```assembly
IMM R1 0        ; Fib(0)
IMM R2 1        ; Fib(1)
LOOP:
  MOV R3 R1
  MOV R4 R2
  ADD R1 R2     ; R1 + R2 → Accumulator
  UADD R5 R0    ; Copy accumulator to R5
  OUT %0 R5     ; Display result
  MOV R1 R2
  MOV R2 R5     ; Next iteration
  JMP LOOP
```

**Behavior:** Generates Fibonacci sequence (0, 1, 1, 2, 3, 5, 8, ...) on port 0.

### 3. Fixed Heart Pattern

```assembly
IMM R1 108      ; 0x6C = 01101100
IMM R2 254      ; 0xFE = 11111110
IMM R3 254
IMM R4 254
IMM R5 124      ; 0x7C = 01111100
IMM R6 56       ; 0x38 = 00111000
IMM R7 16       ; 0x10 = 00010000
OUT %0 R1
OUT %1 R2
OUT %2 R3
OUT %3 R4
OUT %4 R5
OUT %5 R6
OUT %6 R7
```

**Behavior:** Displays a heart pattern on the 8×8 grid (fixed graphic).

---

## Summary

The **Electron CPU** combines a simple yet powerful instruction set with a predictable 4-stage pipeline. Programming requires understanding:

- **Pipeline latency** — Results take 4 cycles to complete
- **Limited resources** — 32 instructions, 7 registers, 8-bit values
- **Conditional logic** — Use flags set by ALU to drive BIE branches
- **Port-based I/O** — Visualize results on an 8×8 LED display

The architecture is **Turing-complete**, meaning any computable function can theoretically be implemented, constrained only by program space and instruction count.
