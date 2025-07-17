

# exeico

A Rust library for working with Portable Executable (PE) files on Windows, built on top of the [`pelite`](https://crates.io/crates/pelite) crate. This project provides the ico data for getting the corresponding id from the PE file

## CLI Usage

```bash
exeico exe <exe_path> <ico_path>           # Extract the main icon from an exe to a .ico file
exeico dll <dll_path> <ico_dir>            # Extract all icons from a dll to .png files in a directory
exeico bin <bin_path> <ico_id> <ico_path>  # Extract an icon by id from a binary to a .ico file
exeico bin <bin_path> <ico_dir>            # Extract all icons from a binary to .ico files in a directory
```

- `exeico exe <exe_path> <ico_path>`: Extract the main icon from an exe file as a .ico file
- `exeico dll <dll_path> <ico_dir>`: Extract all icons from a dll as .png files, saved to the specified directory
- `exeico bin <bin_path> <ico_id> <ico_path>`: Extract an ico by id from a binary file
- `exeico bin <bin_path> <ico_dir>`: Extract all ico files to the specified directory

### Example

```bash
exeico exe C:\Windows\System32\notepad.exe notepad.ico
exeico dll C:\Windows\System32\shell32.dll icons/
exeico bin myapp.exe 0 icon0.ico
exeico bin myapp.exe icons/
```

## Library Usage

Add `exeico` to your `Cargo.toml`:

```toml
[dependencies]
exeico = "0.1.0"
```

```rust
let id = id.parse::<i32>().expect("Invalid icon ID").abs() as u32;
let bin = std::fs::read(exe_path).expect("Failed to read EXE file");
let ico = get_ico(&bin, id).expect("Failed to get icon");
std::fs::write(ico_path, ico).expect("Failed to write icon file");
```
