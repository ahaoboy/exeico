use pelite::pe::Pe as _;
use pelite::pe32;
use pelite::pe32::Pe as _;
use pelite::pe64;
use pelite::resources::Name;
use std::io::Cursor;
use std::io::{Read, Seek, SeekFrom};

fn is64(bin: &[u8]) -> Option<bool> {
    let mut file = Cursor::new(bin);

    // read DOS header offset PE header （e_lfanew）
    file.seek(SeekFrom::Start(0x3C)).ok()?;
    let mut e_lfanew_bytes = [0u8; 4];
    file.read_exact(&mut e_lfanew_bytes).ok()?;
    let e_lfanew = u32::from_le_bytes(e_lfanew_bytes);

    // move to PE header
    file.seek(SeekFrom::Start(e_lfanew as u64)).ok()?;
    let mut signature = [0u8; 4];
    file.read_exact(&mut signature).ok()?;
    if &signature != b"PE\0\0" {
        return None;
    }

    // Skip FileHeader (20 bytes), move to OptionalHeader
    file.seek(SeekFrom::Current(20)).ok()?;

    // read Optional Header Magic 2 bytes
    let mut magic = [0u8; 2];
    file.read_exact(&mut magic).ok()?;
    let magic = u16::from_le_bytes(magic);

    match magic {
        0x10b => Some(false),
        0x20b => Some(true),
        _ => None,
    }
}

pub fn get_ico(bin: &[u8], ico_id: u32) -> Option<Vec<u8>> {
    if is64(bin) == Some(true) {
        let pe = pe64::PeFile::from_bytes(bin).ok()?;

        let resources = pe.resources().ok()?;
        let group_icon_data = resources.icons().flat_map(|i| i.ok());

        for (name, res) in group_icon_data {
            if name == Name::Id(ico_id) {
                let mut v = vec![];
                res.write(&mut v).ok()?;
                return Some(v);
            }
        }
    } else {
        let pe = pe32::PeFile::from_bytes(bin).ok()?;

        let resources = pe.resources().ok()?;
        let group_icon_data = resources.icons().flat_map(|i| i.ok());

        for (name, res) in group_icon_data {
            if name == Name::Id(ico_id) {
                let mut v = vec![];
                res.write(&mut v).ok()?;
                return Some(v);
            }
        }
    };

    None
}