use image::ImageBuffer;
use image::RgbaImage;
use std::io::Cursor;
use std::path::Path;
use widestring::U16CString;
use windows::Win32::Graphics::Gdi::BITMAPINFO;
use windows::Win32::Graphics::Gdi::BITMAPINFOHEADER;
use windows::Win32::Graphics::Gdi::CreateCompatibleDC;
use windows::Win32::Graphics::Gdi::DIB_RGB_COLORS;
use windows::Win32::Graphics::Gdi::DeleteDC;
use windows::Win32::Graphics::Gdi::DeleteObject;
use windows::Win32::Graphics::Gdi::GetDIBits;
use windows::Win32::Graphics::Gdi::SelectObject;
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;
use windows::Win32::UI::WindowsAndMessaging::GetIconInfoExW;
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::Win32::UI::WindowsAndMessaging::ICONINFOEXW;
use windows::core::PCWSTR;

/// Generic icon extraction function that supports extracting single or multiple icons
fn get_images_from_exe(executable_path: &str, id: Option<i32>) -> anyhow::Result<Vec<RgbaImage>> {
    unsafe {
        let path_cstr = U16CString::from_str(executable_path)
            .map_err(|e| anyhow::anyhow!("Failed to convert path to U16CString: {e}"))?;
        let path_pcwstr = PCWSTR(path_cstr.as_ptr());

        let num_icons_total = match id {
            Some(_) => 1,
            None => ExtractIconExW(path_pcwstr, -1, None, None, 0),
        };

        if num_icons_total == 0 {
            return Ok(vec![]); // No icons extracted
        }

        let mut large_icons = vec![HICON::default(); num_icons_total as usize];
        let mut small_icons = vec![HICON::default(); num_icons_total as usize];
        let num_icons_fetched = ExtractIconExW(
            path_pcwstr,
            id.unwrap_or(0),
            Some(large_icons.as_mut_ptr()),
            Some(small_icons.as_mut_ptr()),
            num_icons_total,
        );

        if num_icons_fetched == 0 {
            return Ok(Vec::new()); // No icons extracted
        }

        let images = large_icons
            .iter()
            .chain(small_icons.iter())
            .filter_map(|icon| convert_hicon_to_rgba_image(icon).ok())
            .collect();

        // Clean up icons
        large_icons
            .iter()
            .chain(small_icons.iter())
            .filter(|icon| !icon.is_invalid())
            .map(|icon| DestroyIcon(*icon))
            .filter_map(|r| r.err())
            .for_each(|e| eprintln!("Failed to destroy icon: {e:?}"));

        Ok(images)
    }
}

fn convert_hicon_to_rgba_image(hicon: &HICON) -> anyhow::Result<RgbaImage> {
    unsafe {
        let mut icon_info = windows::Win32::UI::WindowsAndMessaging::ICONINFOEXW {
            cbSize: std::mem::size_of::<ICONINFOEXW>() as u32,
            ..Default::default()
        };
        if !GetIconInfoExW(*hicon, &mut icon_info).as_bool() {
            anyhow::bail!(
                "Failed to get icon info: {} {}:{}",
                file!(),
                line!(),
                column!()
            );
        }

        let hdc_screen = CreateCompatibleDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let hbm_old = SelectObject(hdc_mem, icon_info.hbmColor.into());

        let mut bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: icon_info.xHotspot as i32 * 2,
                biHeight: -(icon_info.yHotspot as i32 * 2),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: DIB_RGB_COLORS.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut buffer: Vec<u8> =
            vec![0; (icon_info.xHotspot * 2 * icon_info.yHotspot * 2 * 4) as usize];

        if GetDIBits(
            hdc_mem,
            icon_info.hbmColor,
            0,
            icon_info.yHotspot * 2,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmp_info,
            DIB_RGB_COLORS,
        ) == 0
        {
            anyhow::bail!("Failed to get DIB bits");
        }

        // Clean up
        SelectObject(hdc_mem, hbm_old);
        let _ = DeleteDC(hdc_mem);
        let _ = DeleteDC(hdc_screen);
        let _ = DeleteObject(icon_info.hbmColor.into());
        let _ = DeleteObject(icon_info.hbmMask.into());

        bgra_to_rgba(buffer.as_mut_slice());

        let image = ImageBuffer::from_raw(icon_info.xHotspot * 2, icon_info.yHotspot * 2, buffer)
            .ok_or_else(|| anyhow::anyhow!("Failed to create ImageBuffer from raw data"))?;
        Ok(image)
    }
}

#[cfg(target_arch = "x86")]
use std::arch::x86::_mm_shuffle_epi8;
use std::arch::x86_64::__m128i;
use std::arch::x86_64::_mm_loadu_si128;
use std::arch::x86_64::_mm_setr_epi8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm_shuffle_epi8;
use std::arch::x86_64::_mm_storeu_si128;

/// Convert BGRA to RGBA
///
/// Uses SIMD to go fast
pub fn bgra_to_rgba(data: &mut [u8]) {
    // The shuffle mask for converting BGRA -> RGBA
    let mask: __m128i = unsafe {
        _mm_setr_epi8(
            2, 1, 0, 3, // First pixel
            6, 5, 4, 7, // Second pixel
            10, 9, 8, 11, // Third pixel
            14, 13, 12, 15, // Fourth pixel
        )
    };
    // For each 16-byte chunk in your data
    for chunk in data.chunks_exact_mut(16) {
        let mut vector = unsafe { _mm_loadu_si128(chunk.as_ptr() as *const __m128i) };
        vector = unsafe { _mm_shuffle_epi8(vector, mask) };
        unsafe { _mm_storeu_si128(chunk.as_mut_ptr() as *mut __m128i, vector) };
    }
}

/// Generic function to convert images to ICO format
fn convert_images_to_ico(images: Vec<RgbaImage>) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut v = vec![];
    for image in images {
        let mut bin = Cursor::new(Vec::new());
        image
            .write_to(&mut bin, image::ImageFormat::Ico)
            .map_err(|e| anyhow::anyhow!("Failed to write image to Ico: {e}"))?;
        v.push(bin.into_inner());
    }
    Ok(v)
}

/// Generic icon extraction function that supports extracting all icons or a single icon
fn extract_icons_from_path<P: AsRef<Path>>(
    path: P,
    id: Option<i32>,
) -> anyhow::Result<Vec<Vec<u8>>> {
    let exe_path = path.as_ref().to_string_lossy();
    let icons = get_images_from_exe(&exe_path, id)?;
    convert_images_to_ico(icons)
}

pub fn get_dll_icos<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Vec<u8>>> {
    extract_icons_from_path(path, None)
}

pub fn get_dll_ico<P: AsRef<Path>>(path: P, id: i32) -> anyhow::Result<Vec<u8>> {
    for i in [id, -id] {
        if let Ok(icons) = extract_icons_from_path(&path, Some(i))
            && let Some(icon) = icons.into_iter().next()
        {
            return Ok(icon);
        }
    }
    Err(anyhow::anyhow!("No icons found in the DLL"))
}

fn extract_text_resource_to_file(dll_path: &str, id: u32) -> anyhow::Result<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::um::libloaderapi::LOAD_LIBRARY_AS_DATAFILE;
    use winapi::um::libloaderapi::LoadStringW;
    use winapi::um::libloaderapi::{
        FindResourceW, FreeLibrary, LoadLibraryExW, LoadResource, LockResource, SizeofResource,
    };
    use winapi::um::winuser::MAKEINTRESOURCEW;

    const RT_RCDATA: i32 = 10;

    fn to_wstring(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(Some(0)).collect()
    }

    let dll_path_w = to_wstring(dll_path);
    let h_module = unsafe {
        LoadLibraryExW(
            dll_path_w.as_ptr(),
            ptr::null_mut(),
            LOAD_LIBRARY_AS_DATAFILE,
        )
    };

    if h_module.is_null() {
        anyhow::bail!("Failed to load library");
    }

    let result = (|| {
        // Attempt to load as string resource
        let mut buffer = [0u16; 4096];
        let len = unsafe { LoadStringW(h_module, id, buffer.as_mut_ptr(), buffer.len() as i32) };

        if len > 0 {
            let text = String::from_utf16(&buffer[..len as usize])
                .map_err(|e| anyhow::anyhow!("UTF-16 decode error: {e}"))?;
            return Ok(text);
        }

        let h_res = unsafe {
            FindResourceW(
                h_module,
                MAKEINTRESOURCEW(id as u16),
                MAKEINTRESOURCEW(RT_RCDATA as u16),
            )
        };
        if h_res.is_null() {
            anyhow::bail!("Resource not found");
        }

        let size = unsafe { SizeofResource(h_module, h_res) };
        let h_data = unsafe { LoadResource(h_module, h_res) };
        let p_data = unsafe { LockResource(h_data) };

        if p_data.is_null() || size == 0 {
            anyhow::bail!("Failed to load or lock resource data");
        }

        let data = unsafe { std::slice::from_raw_parts(p_data as *const u8, size as usize) };
        Ok(String::from_utf8_lossy(data).to_string())
    })();

    unsafe {
        FreeLibrary(h_module);
    }
    result
}

pub fn get_dll_txt<P: AsRef<Path>>(path: P, id: i32) -> anyhow::Result<String> {
    let s = path.as_ref().to_string_lossy();
    let txt = extract_text_resource_to_file(&s, id.unsigned_abs())?;
    Ok(txt)
}
