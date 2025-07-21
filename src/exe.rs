use anyhow::Ok;
use base64::{Engine, engine::general_purpose};
use windows::Win32::System::Threading::CREATE_NO_WINDOW;
use std::{path::Path, process::Command};
use std::os::windows::process::CommandExt;

pub fn get_exe_ico<P: AsRef<Path>>(exe_path: P) -> anyhow::Result<Vec<u8>> {
    let s = exe_path.as_ref().to_string_lossy();
    let pwsh = format!(
        r#"Add-Type -AssemblyName System.Drawing; $icon=[System.Drawing.Icon]::ExtractAssociatedIcon('{s}'); $ms = New-Object System.IO.MemoryStream; $icon.Save($ms); [Convert]::ToBase64String($ms.ToArray())"#
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &pwsh])
        .creation_flags(CREATE_NO_WINDOW.0)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let icon_bytes = general_purpose::STANDARD.decode(stdout.trim())?;
    Ok(icon_bytes)
}
