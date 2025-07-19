use anyhow::{Context, Result};
use exeico::{error, get_dll_ico, get_dll_icos, get_dll_txt, get_exe_ico, get_ico, get_icos};
use std::path::PathBuf;

/// Generic file writing function
fn write_file<P: AsRef<std::path::Path>>(path: P, data: &[u8]) -> Result<()> {
    std::fs::write(&path, data).map_err(|e| error::file_operation_error("Write", &path, e))?;
    Ok(())
}

/// Generic directory creation function
fn create_directory<P: AsRef<std::path::Path>>(dir: P) -> Result<()> {
    std::fs::create_dir_all(&dir)
        .map_err(|e| error::file_operation_error("Create directory", &dir, e))?;
    Ok(())
}

/// Generic binary file reading function
fn read_binary_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<u8>> {
    std::fs::read(&path).map_err(|e| error::file_operation_error("Read", &path, e))
}

/// Generic logic for extracting icons to a single file
fn extract_single_icon<F>(
    extract_fn: F,
    source_path: &str,
    output_path: &str,
    description: &str,
) -> Result<()>
where
    F: FnOnce() -> Result<Vec<u8>>,
{
    let icon_data = extract_fn()?;
    write_file(output_path, &icon_data)?;
    println!("{description} from {source_path} to {output_path}");
    Ok(())
}

/// Generic logic for extracting multiple icons to a directory
fn extract_multiple_icons<F, G>(
    extract_fn: F,
    source_path: &str,
    output_dir: &str,
    filename_fn: G,
    description: &str,
) -> Result<()>
where
    F: FnOnce() -> Result<Vec<Vec<u8>>>,
    G: Fn(usize) -> String,
{
    let icons = extract_fn()?;
    create_directory(output_dir)?;

    for (i, icon_data) in icons.iter().enumerate() {
        let mut out_path = PathBuf::from(output_dir);
        out_path.push(filename_fn(i));
        write_file(&out_path, icon_data)?;
    }

    println!(
        "{} {} icons from {} to {}",
        description,
        icons.len(),
        source_path,
        output_dir
    );
    Ok(())
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match (
        args.next().as_deref(),
        args.next(),
        args.next(),
        args.next(),
    ) {
        // exeico exe <exe_path> <ico_path>
        (Some("exe"), Some(exe_path), Some(ico_path), None) => {
            extract_single_icon(
                || get_exe_ico(&exe_path).context("Failed to get exe icon"),
                &exe_path,
                &ico_path,
                "Extracted main icon",
            )?;
        }
        // exeico dll <dll_path> <ico_dir>
        (Some("dll"), Some(dll_path), Some(ico_dir), None) => {
            extract_multiple_icons(
                || get_dll_icos(&dll_path).context("Failed to get dll icons"),
                &dll_path,
                &ico_dir,
                |i| format!("{i}.png"),
                "Extracted",
            )?;
        }
        (Some("dll-txt"), Some(dll_path), Some(id), None) => {
            let id: i32 = id.parse().map_err(|_| error::parse_error("id", id))?;
            let txt = get_dll_txt(&dll_path, id).context("Failed to get dll txt")?;
            println!("{txt}");
        }
        (Some("dll-ico"), Some(dll_path), Some(id), Some(ico_path)) => {
            let id: i32 = id.parse().map_err(|_| error::parse_error("id", id))?;
            extract_single_icon(
                || get_dll_ico(&dll_path, id).context("Failed to get dll ico"),
                &dll_path,
                &ico_path,
                "Extracted DLL icon",
            )?;
        }
        // exeico bin <bin_path> <ico_id> <ico_path>
        (Some("bin"), Some(bin_path), Some(id), Some(ico_path)) => {
            let id = id
                .parse::<i32>()
                .map_err(|_| error::parse_error("icon ID", id))?;
            let bin = read_binary_file(&bin_path)?;
            extract_single_icon(
                || get_ico(&bin, id).context("Failed to get icon"),
                &bin_path,
                &ico_path,
                &format!("Extracted icon {id}"),
            )?;
        }
        // exeico bin <bin_path> <ico_dir>
        (Some("bin"), Some(bin_path), Some(ico_dir), None) => {
            let bin = read_binary_file(&bin_path)?;
            let icos = get_icos(&bin).context("Failed to get icons")?;
            create_directory(&ico_dir)?;

            for ico in &icos {
                let mut out_path = PathBuf::from(&ico_dir);
                out_path.push(format!("{}.ico", ico.id));
                write_file(&out_path, &ico.data)?;
            }

            println!(
                "Extracted {} icons from {} to {}",
                icos.len(),
                bin_path,
                ico_dir
            );
        }
        _ => {
            println!("Usage:");
            println!(
                "  exeico exe <exe_path> <ico_path>         # Extract main icon from exe to .ico"
            );
            println!(
                "  exeico dll <dll_path> <ico_dir>          # Extract all icons from dll to .png"
            );
            println!(
                "  exeico bin <bin_path> <ico_id> <ico_path> # Extract icon by id from binary to .ico"
            );
            println!(
                "  exeico bin <bin_path> <ico_dir>           # Extract all icons from binary to .ico"
            );
        }
    }
    Ok(())
}
