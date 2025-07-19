

# Exeico

A Rust library and CLI tool for extracting icons from Windows Portable Executable (PE) files. Built on top of the [`pelite`](https://crates.io/crates/pelite) crate, Exeico provides efficient icon extraction capabilities for both executable files and dynamic link libraries.

## Installation

### From Source

```bash
git clone https://github.com/your-repo/exeico.git
cd exeico
cargo build --release
```

### From Cargo

```bash
cargo install exeico
```

## CLI Usage

Exeico provides a command-line interface for extracting icons from various file types:

### Basic Commands

```bash
# Extract main icon from executable
exeico exe <exe_path> <ico_path>

# Extract specific icon by ID from binary
exeico bin <bin_path> <ico_id> <ico_path>

# Extract all icons from binary
exeico bin <bin_path> <ico_dir>

# Extract all icons from DLL
exeico dll <dll_path> <ico_dir>

# Extract text resources from DLL
exeico dll-txt <dll_path> <resource_id>

# Extract specific icon from DLL by ID
exeico dll-ico <dll_path> <icon_id> <ico_path>
```

### Examples

```bash
# Extract Notepad's icon
exeico exe "C:/Windows/System32/notepad.exe" notepad.ico

# Extract all icons from shell32.dll
exeico dll "C:/Windows/System32/shell32.dll" icons/

# Extract icon with ID 0 from custom application
exeico bin "C:/Windows/System32/UserAccountControlSettings.exe" 100 uacs.ico

# Extract all icons from custom application
exeico bin "C:/Windows/System32/UserAccountControlSettings.exe" icons

# Extract text resource with ID 1 from DLL
exeico dll-txt "C:/Windows/System32/shell32.dll" 30312

# Extract specific icon from DLL
exeico dll-ico "C:/WINDOWS/system32/imageres.dll" 109 "./dll-ico.ico"
```

## Library Usage

Add `exeico` to your `Cargo.toml`:

```toml
[dependencies]
exeico = "0.1"
```

### Basic Icon Extraction

```rust
use exeico::{get_ico, get_icos, get_exe_ico, get_dll_icos};
use anyhow::Result;

fn main() -> Result<()> {
    // Extract specific icon from binary data
    let binary_data = std::fs::read("myapp.exe")?;
    let icon_data = get_ico(&binary_data, 0)?;
    std::fs::write("icon0.ico", icon_data)?;

    // Extract all icons from binary data
    let icons = get_icos(&binary_data)?;
    for icon in icons {
        std::fs::write(format!("icon_{}.ico", icon.id), icon.data)?;
    }

    // Extract main icon from executable
    let exe_icon = get_exe_ico("myapp.exe")?;
    std::fs::write("main_icon.ico", exe_icon)?;

    // Extract all icons from DLL
    let dll_icons = get_dll_icos("mylib.dll")?;
    for (i, icon_data) in dll_icons.iter().enumerate() {
        std::fs::write(format!("dll_icon_{}.png", i), icon_data)?;
    }

    Ok(())
}
```

### Advanced Usage

```rust
use exeico::{get_dll_ico, get_dll_txt, error};
use anyhow::Result;

fn extract_resources() -> Result<()> {
    // Extract specific icon from DLL
    let icon_data = get_dll_ico("mylib.dll", 1)?;
    std::fs::write("specific_icon.png", icon_data)?;

    // Extract text resource from DLL
    let text = get_dll_txt("mylib.dll", 100)?;
    println!("Extracted text: {}", text);

    // Handle errors gracefully
    match get_dll_ico("nonexistent.dll", 0) {
        Ok(data) => println!("Icon extracted successfully"),
        Err(e) => eprintln!("Failed to extract icon: {}", e),
    }

    Ok(())
}
```

## API Reference

### Core Functions

- `get_ico(bin: &[u8], ico_id: i32) -> Result<Vec<u8>>` - Extract specific icon from binary data
- `get_icos(bin: &[u8]) -> Result<Vec<Ico>>` - Extract all icons from binary data
- `get_exe_ico<P: AsRef<Path>>(path: P) -> Result<Vec<u8>>` - Extract main icon from executable
- `get_dll_icos<P: AsRef<Path>>(path: P) -> Result<Vec<Vec<u8>>>` - Extract all icons from DLL
- `get_dll_ico<P: AsRef<Path>>(path: P, id: i32) -> Result<Vec<u8>>` - Extract specific icon from DLL
- `get_dll_txt<P: AsRef<Path>>(path: P, id: i32) -> Result<String>` - Extract text resource from DLL

### Data Structures

```rust
pub struct Ico {
    pub id: String,    // Icon identifier
    pub data: Vec<u8>, // Icon data in ICO format
}
```
