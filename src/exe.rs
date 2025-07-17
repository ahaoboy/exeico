use anyhow::Ok;
use base64::{Engine, engine::general_purpose};
use std::process::Command;

pub fn get_exe_ico(exe_path: &str) -> anyhow::Result<Vec<u8>> {
    let pwsh = format!(
        r#"Add-Type -AssemblyName System.Drawing; $icon=[System.Drawing.Icon]::ExtractAssociatedIcon('{exe_path}'); $ms = New-Object System.IO.MemoryStream; $icon.Save($ms); [Convert]::ToBase64String($ms.ToArray())"#
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &pwsh])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let icon_bytes = general_purpose::STANDARD.decode(stdout.trim())?;
    Ok(icon_bytes)
}
