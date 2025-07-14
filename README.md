

# exeico

A Rust library for working with Portable Executable (PE) files on Windows, built on top of the [`pelite`](https://crates.io/crates/pelite) crate. This project provides the ico data for getting the corresponding id from the PE file

## Usage

```bash
exeico <exe_path> <id> <ico_path>


```
### Example

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
