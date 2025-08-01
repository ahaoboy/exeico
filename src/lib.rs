use anyhow::{Context, Result};
use pelite::resources::Name;
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

/// Generic PE file processing function that calls the appropriate handler based on file type
fn with_pe_file<F, T>(bin: &[u8], f: F) -> Result<T>
where
    F: FnOnce(&pelite::PeFile<'_>) -> Result<T>,
{
    let pe = pelite::PeFile::from_bytes(bin).context("Failed to parse PE file")?;
    f(&pe)
}

pub fn get_ico(bin: &[u8], ico_id: i32) -> Result<Vec<u8>> {
    with_pe_file(bin, |pe| extract_ico(pe, ico_id))
}

fn extract_ico(pe: &pelite::PeFile, ico_id: i32) -> Result<Vec<u8>> {
    let resources = pe.resources().context("No resources found in PE file")?;
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
    with_pe_file(bin, extract_icos)
}

pub struct Ico {
    pub id: String,
    pub data: Vec<u8>,
}

fn extract_icos(pe: &pelite::PeFile) -> Result<Vec<Ico>> {
    let resources = pe
        .resources()
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
