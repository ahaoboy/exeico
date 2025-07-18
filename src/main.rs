use anyhow::{Context, Result};
use exeico::{get_dll_ico, get_dll_icos, get_dll_txt, get_exe_ico, get_ico, get_icos};
use std::path::PathBuf;

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
            let ico = get_exe_ico(&exe_path).context("Failed to get exe icon")?;
            std::fs::write(&ico_path, ico)
                .with_context(|| format!("Failed to write icon file: {ico_path}"))?;
            println!("Extracted main icon from {exe_path} to {ico_path}");
        }
        // exeico dll <dll_path> <ico_dir>
        (Some("dll"), Some(dll_path), Some(ico_dir), None) => {
            let icons = get_dll_icos(&dll_path).context("Failed to get dll icons")?;
            std::fs::create_dir_all(&ico_dir)
                .with_context(|| format!("Failed to create directory: {ico_dir}"))?;
            for (i, icon) in icons.iter().enumerate() {
                let mut out_path = PathBuf::from(&ico_dir);
                out_path.push(format!("{i}.png"));
                std::fs::write(&out_path, icon)
                    .with_context(|| format!("Failed to write icon file: {out_path:?}"))?;
            }
            println!(
                "Extracted {} icons from {} to {}",
                icons.len(),
                dll_path,
                ico_dir
            );
        }
        (Some("dll-txt"), Some(dll_path), Some(id), None) => {
            let id: i32 = id.parse().expect("Failed to parse id");
            let txt = get_dll_txt(&dll_path, id).context("Failed to get dll txt")?;
            println!("{txt}");
        }
        (Some("dll-ico"), Some(dll_path), Some(id), Some(ico_path)) => {
            let id: i32 = id.parse().expect("Failed to parse id");
            let ico = get_dll_ico(&dll_path, id).expect("Failed to get dll ico");
            std::fs::write(ico_path, ico)?;
        }
        // exeico bin <bin_path> <ico_id> <ico_path>
        (Some("bin"), Some(bin_path), Some(id), Some(ico_path)) => {
            let id = id.parse::<i32>().context("Invalid icon ID")?;
            let bin = std::fs::read(&bin_path)
                .with_context(|| format!("Failed to read binary file: {bin_path}"))?;
            let ico = get_ico(&bin, id).context("Failed to get icon")?;
            std::fs::write(&ico_path, ico)
                .with_context(|| format!("Failed to write icon file: {ico_path}"))?;
            println!("Extracted icon {id} from {bin_path} to {ico_path}");
        }
        // exeico bin <bin_path> <ico_dir>
        (Some("bin"), Some(bin_path), Some(ico_dir), None) => {
            let bin = std::fs::read(&bin_path)
                .with_context(|| format!("Failed to read binary file: {bin_path}"))?;
            let icos = get_icos(&bin).context("Failed to get icons")?;
            std::fs::create_dir_all(&ico_dir)
                .with_context(|| format!("Failed to create directory: {ico_dir}"))?;
            for ico in &icos {
                let mut out_path = PathBuf::from(&ico_dir);
                out_path.push(format!("{}.ico", ico.id));
                std::fs::write(&out_path, &ico.data)
                    .with_context(|| format!("Failed to write icon file: {out_path:?}"))?;
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
