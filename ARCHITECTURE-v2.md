# Electron V2 Architecture & Manual

This system is a faithful emulation of **Electron 2**, the successor to the original Electron CPU. It is currently under active construction in Minecraft (PS5 Edition) to push the limits of redstone computing.

Electron, the original CPU (which features a simpler architecture but is fully documented), can be found in the [ARCHITECTURE.md](ARCHITECTURE.md) file.

## Specifications

*   **Registers:** 7 General Purpose (R1-R7) + 1 Zero Register (R0).
*   **Memory:** 16 Bytes of RAM (Shared with Stack).
*   **Display:** 8x8 Pixel Grid (Mapped to 8 Ports, 8 bits each).
*   **ROM:** 256 Lines of Program Memory.

## Syntax & Formatting

*   **Case Insensitive:** Operations and operands can be uppercase or lowercase (e.g., `MOV`, `mov`, `R1`, `r1`).
*   **Registers:** Can be prefixed with `R` or `$` (e.g., `R1`, `$1`).
*   **Memory Addresses:** Can be prefixed with `#` or `@` (e.g., `#10`, `@5`).
*   **I/O Ports:** Must be prefixed with `%` (e.g., `%0`).
*   **Binary Numbers:** Can be prefixed with `B` or `b` (e.g., `B101`, `b101`).
*   **Operands:** Operands A and B can be Registers (R0-R7), Numbers (0-255), or Ports (%0-%7).

## Hardware Constraints

**⚠ Pipeline & Latency**

This architecture utilizes a raw pipeline without hardware interlocking.

**Read-After-Write Latency:**
Registers generally update in the WriteBack (Final) stage. Reading a register immediately after writing it usually yields the OLD value. You may need to insert `NOOP` instructions or unrelated operations to wait for the write to complete before reading.

## Instruction Set

### Assignments

| Syntax | Description |
| :--- | :--- |
| `IMM A B` | Set Register A to value B. |
| `MOV A B` | Copy value from Register B to A. |

### Math & Logic

| Syntax | Description |
| :--- | :--- |
| `ADD A B` | A = A + B |
| `ADDC A B` | A = A + B + Overflow Flag (from prev op) |
| `SUB A B` | A = A - B |
| `AND A B` | A = A & B (Bitwise AND) |
| `OR A B` | A = A \| B (Bitwise OR) |
| `XOR A B` | A = A ^ B (Bitwise XOR) |
| `SHR A B` | A = B shifted right by 1. |
| `NOT A B` | A = Inverted bits of B |

### ALU Prefixes (U, X)

Prefix any Math or Logic op (`ADD`, `SUB`, `XOR`, etc) to change the operands.

| Prefix | Example | Behavior |
| :--- | :--- | :--- |
| **-** | `ADD A B` | **A = A + B** (Standard. Updates A & ACC.) |
| **U** | `UADD A B` | **A = ACC + B** (Chain calculation. Uses ACC input.) |
| **X** | `XADD A B` | **ACC = ACC + B** (Compare only. A is unchanged.) |

### Memory (RAM)

| Syntax | Description |
| :--- | :--- |
| `STORE A B` | Save Register B into Memory Address A. |
| `LOAD A B` | Load Memory Address B into Register A. |

### Flow Control

| Syntax | Description |
| :--- | :--- |
| `JMP A` | Jump to Line Number A. |
| `BIE A` | Jump to A if Equal (==). |
| `BIG A` | Jump to A if Greater (>). |
| `BIL A` | Jump to A if Less (<). |
| `BIO A` | Jump to A if Overflow. |
| `CALL A` | Run Function at Line A. |
| `RET` | Return from function. |

### System & I/O

| Syntax | Description |
| :--- | :--- |
| `OUT A B` | Send Register B to Port A (%0-%7). |
| `ROUT A B` | Send Register B to Port in Register A (%0-%7). |
| `INP A` | Wait for user input, store in Register A. |
| `PUSH A` | Push Register A onto Stack. |
| `POP A` | Pop Stack into Register A. |
| `NOOP` | No Operation (Do nothing). |

## Pro Tips & Patterns

### Non-Destructive Compare
Use the **X** prefix to compare registers without overwriting them.
`XSUB R0 R2` subtracts R2 from the Accumulator (loaded from previous op) but discards the result, setting only the flags for branching.

### The Zero Register (R0)
R0 is hardwired to 0. Use it as a source for clearing registers (`MOV R1 R0`) or for comparisons (`SUB R1 R0` checks if R1 is 0).

### Multiplication
There is no `MUL` instruction. You must implement multiplication via repeated addition loops.

## Compilation & Validation

*   **ERRORS:** Syntax errors, invalid mnemonics, or illegal characters.
*   **WARNINGS:** Code that is technically valid but may result in unintended behavior, such as writing to R0, memory/port out-of-bounds, or pipeline hazards.
