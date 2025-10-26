# Electron Redstone CPU

This is a complete computer-architecture toolchain built from scratch in Rust. It is centered around **Electron**, a custom 8-bit, Turing-complete, RISC-V-inspired CPU.

This repository contains all components of the toolchain:

* **The Assembler:** A compiler that translates custom Electron Assembly (`.elt`) into 8-bit machine code.
* **The Emulator:** A Rust-based emulator with a `raylib-rs` GUI that runs the machine code, allowing for rapid development and debugging.
* **The VS Code Extension:** Provides full syntax highlighting for the Electron Assembly language.
* **The Minecraft Implementation:** The final CPU design was prototyped and built using Redstone components in Minecraft, proving the architecture in a sandboxed, logic-gate environment.

## Minecraft Redstone Implementation

As a proof-of-concept, the final Electron V1 CPU was designed and built from first principles using Redstone components within Minecraft. This demonstrates the design in a sandboxed, low-level logic-gate environment.

![Screenshot of the Electron CPU built in Minecraft](minecraft.png)

### CPU Specifications
* **ROM:** 96 bytes (32 lines)
* **Instruction size:** 3 bytes
* **RAM:** 32 bytes
* **ALUP:** 8 ticks
* **Registers:** 7 bytes (+1 zero register)
* **Cores:** 1
* **Pipeline:** 4 stage waterfall
* **Bus width:** 8 bit
* **Display:** 8x8 screen
* **I/O:** 8 Bytes out, 1 byte in
* **Speed (Real time):** 1hz

---

## Running the Toolchain

You can compile and run an assembly program in the Rust-based emulator with a single command. This will build the toolchain, assemble the `.elt` file, and launch the emulator GUI.

```sh
# Assemble and run the fibonacci program in the emulator
cargo run -- -f fibonacci.elt

# Assemble and run the heart-drawing program
cargo run -- -f heart.elt

# Or you can run without compiling (Easiest):
./electron -f heart.elt
```

## Emulator GUI

The emulator provides a visual interface to inspect the CPU's state, including registers, RAM, and the 8x8 display output.

![Screenshot of the Electron CPU built in Minecraft](gui.png)

## Prerequisites

The emulator GUI is built with raylib-rs. To run it, you may need to install C/C++ dependencies from raylib-rs.

## The Electron ISA & Tooling

The custom 8-bit Instruction Set Architecture (ISA) is **Turing-complete** and **inspired by RISC-V** principles.

For a comprehensive technical deep-dive into the CPU architecture, instruction set, pipeline model, and programming guidelines, see the [**ARCHITECTURE.md**](ARCHITECTURE.md) documentation.

### Supported Instructions
It currently supports the following instructions and their variants:

- `IMM`
- `MOV`
- `ADD`
- `ADDC`
- `SHR`
- `NOOP`
- `OUT`
- `JMP`
- `BIE`
- `NOT`

### Binary Literal Format

When writing binary literals in assembly, use the **uppercase `B` prefix**:
```assembly
IMM R1 B11110000

```

### VS Code Extension

To make programming easier, a custom VS Code extension provides full syntax highlighting for `.elt` files.

**Quick Installation:**

1. Open VS Code
2. Press `Ctrl+Shift+X` (or `Cmd+Shift+X` on macOS) to open Extensions
3. Click the `...` menu → Select **"Install from VSIX..."**
4. Navigate to `electron-lang/electron-language-0.0.1.vsix` and open it
5. Reload VS Code when prompted

**For detailed installation instructions** (including alternative methods and troubleshooting), see [`electron-lang/README.md`](electron-lang/README.md).

![Syntax Highlighting Example](https://github.com/user-attachments/assets/a1841e33-3296-4aee-bc1d-d63cdf80b4d8)


## Additional Resources

For more detailed information, refer to the [spreadsheet](https://docs.google.com/spreadsheets/d/1BrFaLE5tVunBa1GLoMH4RvVh4GXqqtaN3Qv29scvfyQ/edit?usp=sharing). Note that this document may not be fully up-to-date with the current progress of the repo.
