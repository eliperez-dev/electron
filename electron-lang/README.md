# Electron Language Syntax Highlighting

A Visual Studio Code extension that provides full syntax highlighting support for the **Electron Assembly Language** (`.elt` files). Syntax highlighting makes it easier to write, debug, and understand Electron programs with color-coded instructions, registers, operands, and comments.

## Features

- ✅ **Syntax highlighting** for all Electron instructions (IMM, MOV, ADD, ADDC, SHR, NOT, OUT, JMP, BIE, NOOP)
- ✅ **Color-coded operands** — Registers (R0–R7), ports (%0–%7), immediates, binary literals
- ✅ **Comment support** — Gray comments for documentation
- ✅ **Binary literal support** — Recognized B-notation (B11001010)
- ✅ **Auto-detection** — Automatically recognizes `.elt` files

## Installation

### Method 1: Install from VSIX File (Recommended)

The easiest way to install the extension is directly from the prebuilt VSIX file.

**Steps:**

1. **Open VS Code**

2. **Open the Extensions sidebar**
   - Press `Ctrl+Shift+X` (or `Cmd+Shift+X` on macOS)
   - Or click the Extensions icon on the left sidebar

3. **Install from VSIX**
   - Click the `...` menu at the top of the Extensions panel
   - Select **"Install from VSIX..."**
   - Navigate to `electron-lang/electron-language-0.0.1.vsix`
   - Click Open

4. **Confirm installation**
   - The extension will install and show as "Installed"
   - You may need to reload VS Code (a reload button should appear)

### Method 2: Manual Installation (File System)

If the VSIX method doesn't work, you can manually copy the extension files:

1. **Locate your VS Code extensions directory:**
   - **Windows:** `%USERPROFILE%\.vscode\extensions`
   - **macOS:** `~/.vscode/extensions`
   - **Linux:** `~/.vscode/extensions`

2. **Copy the extension folder:**
   ```bash
   cp -r electron-lang ~/.vscode/extensions/electron-language-0.0.1
   ```
   (Adjust path as needed for your OS)

3. **Reload VS Code**
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS)
   - Type "Developer: Reload Window"
   - Press Enter

### Method 3: Build and Install from Source

If you want to build the extension yourself:

1. **Install the VSCE tool** (VS Code Extension Manager)
   ```bash
   npm install -g vsce
   ```

2. **Navigate to the extension directory**
   ```bash
   cd electron-lang
   ```

3. **Build the VSIX package**
   ```bash
   vsce package
   ```

4. **Install the newly built VSIX**
   - Follow Method 1 above with the generated `.vsix` file

## Usage

Once installed:

1. **Open or create an `.elt` file** in VS Code
2. **Syntax highlighting automatically applies**
3. **Start writing Electron Assembly code**

### Example

```assembly
; Fire effect program
IMM R1 B11111111  ; Load all-ones into R1
OUT %0 R1         ; Display on port 0
JMP 0             ; Loop back
```

The extension will color-code:
- **Instructions** (IMM, OUT, JMP) in one color
- **Registers** (R1) in another color
- **Immediates** (B11111111) in another color
- **Comments** in gray

## Supported File Type

- **`.elt`** — Electron Assembly Language source files

When you open a `.elt` file, VS Code automatically recognizes it and applies the Electron language syntax highlighting.

## Language Configuration

The extension includes language configuration for:
- **Comment syntax:** Lines starting with `;`
- **Bracket matching:** Auto-recognition of operand boundaries
- **Auto-indentation:** Proper formatting for nested blocks

See `language-configuration.json` for details.

## Syntax Grammar

The syntax highlighting rules are defined in `syntaxes/electron.tmLanguage.json`, which follows the TextMate grammar format used by VS Code.

## Troubleshooting

### Syntax highlighting not appearing

1. **Ensure the file extension is `.elt`**
   - Right-click the file tab → Select Language Mode → Search "Electron"

2. **Manually set the language mode**
   - Click the language selector in the bottom-right of VS Code
   - Type "Electron" and select it

3. **Reload VS Code**
   - Press `Ctrl+Shift+P` → Type "Reload Window"

### Extension won't install

1. **Check VS Code version**
   - The extension requires VS Code 1.50.0 or later
   - Check your version in **Help → About**

2. **Try Method 2 (Manual Installation)**
   - Copy files directly to `.vscode/extensions`

3. **Reinstall the extension**
   - Uninstall: Extensions panel → Right-click → Uninstall
   - Delete the extension folder from `~/.vscode/extensions`
   - Reinstall using Method 1

## Contributing

To modify the syntax highlighting:

1. Edit `syntaxes/electron.tmLanguage.json` for highlighting rules
2. Edit `language-configuration.json` for language settings
3. Rebuild with `vsce package`
4. Test in VS Code

## Version

**Current Version:** 0.0.1

## License

See `LICENSE.md` for license information.