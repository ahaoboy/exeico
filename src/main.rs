use anyhow::{Context, Result};
use exeico::{get_ico, get_icos};
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match (args.next(), args.next(), args.next()) {
        (Some(exe_path), Some(second), None) => {
            let bin = std::fs::read(&exe_path)
                .with_context(|| format!("Failed to read EXE file: {exe_path}"))?;
            let icos = get_icos(&bin).context("Failed to get icons")?;
            std::fs::create_dir_all(&second)
                .with_context(|| format!("Failed to create directory: {second}"))?;
            for ico in &icos {
                let mut out_path = PathBuf::from(&second);
                out_path.push(format!("{}.ico", ico.id));
                std::fs::write(&out_path, &ico.data)
                    .with_context(|| format!("Failed to write icon file: {out_path:?}"))?;
            }
            println!("Extracted {} icons to {}", icos.len(), second);
        }
        (Some(exe_path), Some(id), Some(ico_path)) => {
            let id = id.parse::<i32>().context("Invalid icon ID")?;
            let bin = std::fs::read(&exe_path)
                .with_context(|| format!("Failed to read EXE file: {exe_path}"))?;
            let ico = get_ico(&bin, id).context("Failed to get icon")?;
            std::fs::write(&ico_path, ico)
                .with_context(|| format!("Failed to write icon file: {ico_path}"))?;
        }
        _ => {
            println!("exeico <exe_path> <id> <ico_path>");
            println!("exeico <exe_path> <ico_dir>");
        }
    }
    Ok(())
}
