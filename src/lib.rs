use anyhow::{Context, Result};
use pelite::pe::Pe as _;
use pelite::pe32;
use pelite::pe32::Pe as _;
use pelite::pe64;
use pelite::resources::Name;
use std::io::Cursor;
use std::io::{Read, Seek, SeekFrom};
mod exe;
pub use exe::*;
mod dll;
pub use dll::*;

/// Common error handling module
pub mod error {
    use anyhow::anyhow;
    use std::path::Path;

    /// Common file operation error handling
    pub fn file_operation_error<P: AsRef<Path>>(
        operation: &str,
        path: P,
        error: std::io::Error,
    ) -> anyhow::Error {
        anyhow!("{} failed for {:?}: {}", operation, path.as_ref(), error)
    }

    /// Common parsing error handling
    pub fn parse_error<T: std::fmt::Display>(item: &str, value: T) -> anyhow::Error {
        anyhow!("Failed to parse {}: {}", item, value)
    }

    /// Common resource not found error handling
    pub fn resource_not_found(resource_type: &str, id: i32) -> anyhow::Error {
        anyhow!("{} with id {} not found", resource_type, id)
    }
}

fn is64(bin: &[u8]) -> Result<bool> {
    let mut file = Cursor::new(bin);

    file.seek(SeekFrom::Start(0x3C))
        .context("Failed to seek to DOS header offset (e_lfanew)")?;
    let mut e_lfanew_bytes = [0u8; 4];
    file.read_exact(&mut e_lfanew_bytes)
        .context("Failed to read e_lfanew bytes")?;
    let e_lfanew = u32::from_le_bytes(e_lfanew_bytes);

    file.seek(SeekFrom::Start(e_lfanew as u64))
        .context("Failed to seek to PE header")?;
    let mut signature = [0u8; 4];
    file.read_exact(&mut signature)
        .context("Failed to read PE signature")?;
    if &signature != b"PE\0\0" {
        anyhow::bail!("Invalid PE signature");
    }

    file.seek(SeekFrom::Current(20))
        .context("Failed to seek to OptionalHeader")?;

    let mut magic = [0u8; 2];
    file.read_exact(&mut magic)
        .context("Failed to read Optional Header magic")?;
    let magic = u16::from_le_bytes(magic);

    match magic {
        0x10b => Ok(false),
        0x20b => Ok(true),
        _ => anyhow::bail!("Unknown PE magic: {magic:#x}"),
    }
}

trait PeResources {
    fn get_resources(&'_ self) -> Option<pelite::resources::Resources<'_>>;
}

impl PeResources for pe32::PeFile<'_> {
    fn get_resources(&'_ self) -> Option<pelite::resources::Resources<'_>> {
        self.resources().ok()
    }
}
impl PeResources for pe64::PeFile<'_> {
    fn get_resources(&'_ self) -> Option<pelite::resources::Resources<'_>> {
        self.resources().ok()
    }
}

/// Generic PE file processing function that calls the appropriate handler based on file type
fn with_pe_file<F, T>(bin: &[u8], f: F) -> Result<T>
where
    F: FnOnce(&dyn PeResources) -> Result<T>,
{
    let is_pe64 = is64(bin)?;

    if is_pe64 {
        let pe = pe64::PeFile::from_bytes(bin).context("Failed to parse PE64 file")?;
        f(&pe)
    } else {
        let pe = pe32::PeFile::from_bytes(bin).context("Failed to parse PE32 file")?;
        f(&pe)
    }
}

pub fn get_ico(bin: &[u8], ico_id: i32) -> Result<Vec<u8>> {
    with_pe_file(bin, |pe| extract_ico(pe, ico_id))
}

fn extract_ico<P: PeResources + ?Sized>(pe: &P, ico_id: i32) -> Result<Vec<u8>> {
    let resources = pe
        .get_resources()
        .context("No resources found in PE file")?;
    let group_icon_data = resources.icons().flat_map(|i| i.ok());
    for (name, res) in group_icon_data {
        if name == Name::Id(ico_id.unsigned_abs()) {
            let mut v = vec![];
            res.write(&mut v).context("Failed to write icon data")?;
            return Ok(v);
        }
    }
    Err(error::resource_not_found("Icon", ico_id))
}

pub fn get_icos(bin: &[u8]) -> Result<Vec<Ico>> {
    with_pe_file(bin, |pe| extract_icos(pe))
}

pub struct Ico {
    pub id: String,
    pub data: Vec<u8>,
}

fn extract_icos<P: PeResources + ?Sized>(pe: &P) -> Result<Vec<Ico>> {
    let resources = pe
        .get_resources()
        .context("No resources found in PE file")?;
    let group_icon_data = resources.icons().flat_map(|i| i.ok());
    let mut v = vec![];
    for (name, res) in group_icon_data {
        let mut buf = vec![];
        res.write(&mut buf).context("Failed to write icon data")?;
        v.push(Ico {
            id: name.to_string(),
            data: buf,
        });
    }
    Ok(v)
}
