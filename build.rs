//! Build script. Puts `lipi.ico` into the exe as its icon, so Explorer, the taskbar,
//! Alt+Tab and the shortcut all show it.
//!
//! No crates and no extra tools (no rc.exe, no windres): the icon file is turned into a
//! Windows resource file here, and the MSVC linker, which is already installed with Rust,
//! adds it to the exe. If `lipi.ico` is missing or broken, the exe is still built, with the
//! default icon, and a warning is shown.

use std::env;
use std::fs;
use std::path::PathBuf;

const RT_ICON: u16 = 3;
const RT_GROUP_ICON: u16 = 14;
/// The icon group Explorer shows for the exe. Only one group exists, so any number would do.
const GROUP_ID: u16 = 1;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=lipi.ico");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return; // other systems have no exe icons
    }
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        println!("cargo:warning=The exe icon is added with the MSVC toolchain only. Building without it.");
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let ico_path = manifest_dir.join("lipi.ico");
    let ico = match fs::read(&ico_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("cargo:warning=Could not read {}: {e}. Building with the default icon.", ico_path.display());
            return;
        }
    };
    let res = match icon_resource(&ico) {
        Ok(res) => res,
        Err(e) => {
            println!("cargo:warning=lipi.ico is not a usable icon file ({e}). Building with the default icon.");
            return;
        }
    };

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let res_path = out_dir.join("lipi.res");
    if let Err(e) = fs::write(&res_path, res) {
        println!("cargo:warning=Could not write {}: {e}. Building with the default icon.", res_path.display());
        return;
    }
    // The linker takes a .res file like any other input and adds it to the exe.
    println!("cargo:rustc-link-arg-bins={}", res_path.display());
}

fn u16_at(data: &[u8], at: usize) -> Result<u16, String> {
    data.get(at..at + 2).map(|b| u16::from_le_bytes([b[0], b[1]])).ok_or_else(|| "file too short".to_string())
}

fn u32_at(data: &[u8], at: usize) -> Result<u32, String> {
    data.get(at..at + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| "file too short".to_string())
}

/// One resource in a .res file: a 32 byte header, the data, and padding up to 4 bytes.
/// Type and name are plain numbers.
fn resource(type_id: u16, name_id: u16, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(32 + data.len() + 3);
    out.extend_from_slice(&(data.len() as u32).to_le_bytes()); // size of the data
    out.extend_from_slice(&32u32.to_le_bytes()); // size of this header
    out.extend_from_slice(&0xFFFFu16.to_le_bytes()); // the next number is a type id
    out.extend_from_slice(&type_id.to_le_bytes());
    out.extend_from_slice(&0xFFFFu16.to_le_bytes()); // the next number is a name id
    out.extend_from_slice(&name_id.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // data version
    out.extend_from_slice(&0x1030u16.to_le_bytes()); // memory flags: movable, pure, discardable
    out.extend_from_slice(&0x0409u16.to_le_bytes()); // language: English (US)
    out.extend_from_slice(&0u32.to_le_bytes()); // version
    out.extend_from_slice(&0u32.to_le_bytes()); // characteristics
    out.extend_from_slice(data);
    while out.len() % 4 != 0 {
        out.push(0);
    }
    out
}

/// Turns the bytes of an .ico file into the bytes of a .res file:
/// one RT_ICON resource for each picture and one RT_GROUP_ICON resource listing them.
fn icon_resource(ico: &[u8]) -> Result<Vec<u8>, String> {
    if u16_at(ico, 0)? != 0 || u16_at(ico, 2)? != 1 {
        return Err("it does not start like an .ico file".into());
    }
    let count = u16_at(ico, 4)? as usize;
    if count == 0 || count > 64 {
        return Err(format!("it holds {count} pictures"));
    }

    // A .res file always starts with an empty resource.
    let mut res = empty_resource();
    for i in 0..count {
        let entry = 6 + 16 * i;
        let size = u32_at(ico, entry + 8)? as usize;
        let offset = u32_at(ico, entry + 12)? as usize;
        let picture = offset
            .checked_add(size)
            .and_then(|end| ico.get(offset..end))
            .ok_or_else(|| format!("picture {} is cut off", i + 1))?;
        if size == 0 {
            return Err(format!("picture {} is empty", i + 1));
        }
        res.extend_from_slice(&resource(RT_ICON, (i + 1) as u16, picture));
    }
    res.extend_from_slice(&resource(RT_GROUP_ICON, GROUP_ID, &group_data(ico, count)?));
    Ok(res)
}

/// The empty first resource every .res file begins with.
fn empty_resource() -> Vec<u8> {
    let mut out = Vec::with_capacity(32);
    out.extend_from_slice(&0u32.to_le_bytes()); // no data
    out.extend_from_slice(&32u32.to_le_bytes()); // header size
    out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&[0u8; 16]); // data version, flags, language, version, characteristics
    out
}

/// The RT_GROUP_ICON data: a 6 byte header, then 14 bytes for each picture:
/// width, height, colours, reserved, planes, bits per pixel (these first 8 bytes are copied
/// from the .ico directory), the size of the picture (4 bytes), and the number of its
/// RT_ICON resource (2 bytes, instead of the position in the file).
fn group_data(ico: &[u8], count: usize) -> Result<Vec<u8>, String> {
    let mut group = Vec::with_capacity(6 + 14 * count);
    group.extend_from_slice(&0u16.to_le_bytes()); // reserved
    group.extend_from_slice(&1u16.to_le_bytes()); // type 1 = icon
    group.extend_from_slice(&(count as u16).to_le_bytes());
    for i in 0..count {
        let entry = 6 + 16 * i;
        group.extend_from_slice(ico.get(entry..entry + 8).ok_or("file too short")?);
        group.extend_from_slice(&u32_at(ico, entry + 8)?.to_le_bytes());
        group.extend_from_slice(&((i + 1) as u16).to_le_bytes());
    }
    Ok(group)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny made-up .ico: two pictures of 5 and 7 bytes.
    fn sample_ico() -> Vec<u8> {
        let mut ico = vec![0, 0, 1, 0, 2, 0];
        let first = [16u8, 16, 0, 0, 1, 0, 32, 0, 5, 0, 0, 0];
        let second = [0u8, 0, 0, 0, 1, 0, 32, 0, 7, 0, 0, 0];
        let data_start = 6 + 32;
        ico.extend_from_slice(&first);
        ico.extend_from_slice(&(data_start as u32).to_le_bytes());
        ico.extend_from_slice(&second);
        ico.extend_from_slice(&((data_start + 5) as u32).to_le_bytes());
        ico.extend_from_slice(b"AAAAA");
        ico.extend_from_slice(b"BBBBBBB");
        ico
    }

    /// Reads a .res file back: (type, name, data) for every resource.
    fn read_res(res: &[u8]) -> Vec<(u16, u16, Vec<u8>)> {
        let mut found = Vec::new();
        let mut at = 0;
        while at < res.len() {
            let size = u32_at(res, at).unwrap() as usize;
            let header = u32_at(res, at + 4).unwrap() as usize;
            assert_eq!(header, 32, "only numeric type and name are written");
            assert_eq!(u16_at(res, at + 8).unwrap(), 0xFFFF);
            assert_eq!(u16_at(res, at + 12).unwrap(), 0xFFFF);
            let type_id = u16_at(res, at + 10).unwrap();
            let name_id = u16_at(res, at + 14).unwrap();
            let data = res[at + header..at + header + size].to_vec();
            found.push((type_id, name_id, data));
            at += header + size;
            at = (at + 3) / 4 * 4;
        }
        assert_eq!(at, res.len(), "the file must end exactly after the last resource");
        found
    }

    #[test]
    fn starts_with_the_empty_resource() {
        let res = icon_resource(&sample_ico()).unwrap();
        assert_eq!(&res[..32], &empty_resource()[..]);
        assert_eq!(&res[..8], &[0, 0, 0, 0, 32, 0, 0, 0]);
    }

    #[test]
    fn writes_one_icon_resource_for_each_picture_and_one_group() {
        let res = icon_resource(&sample_ico()).unwrap();
        let found = read_res(&res);
        assert_eq!(found.len(), 4); // empty, two pictures, group
        assert_eq!((found[0].0, found[0].1, found[0].2.len()), (0, 0, 0));
        assert_eq!((found[1].0, found[1].1), (RT_ICON, 1));
        assert_eq!(found[1].2, b"AAAAA");
        assert_eq!((found[2].0, found[2].1), (RT_ICON, 2));
        assert_eq!(found[2].2, b"BBBBBBB");
        assert_eq!((found[3].0, found[3].1), (RT_GROUP_ICON, GROUP_ID));
    }

    #[test]
    fn the_group_lists_each_picture_with_its_resource_number() {
        let found = read_res(&icon_resource(&sample_ico()).unwrap());
        let group = &found[3].2;
        assert_eq!(group.len(), 6 + 14 * 2);
        assert_eq!(&group[..6], &[0, 0, 1, 0, 2, 0]);
        // entry 1: 16x16, 32 bit, 5 bytes, resource 1
        assert_eq!(&group[6..20], &[16, 16, 0, 0, 1, 0, 32, 0, 5, 0, 0, 0, 1, 0]);
        // entry 2: the 256 size is stored as 0, 7 bytes, resource 2
        assert_eq!(&group[20..34], &[0, 0, 0, 0, 1, 0, 32, 0, 7, 0, 0, 0, 2, 0]);
    }

    #[test]
    fn every_resource_starts_on_a_four_byte_boundary() {
        let res = icon_resource(&sample_ico()).unwrap();
        assert_eq!(res.len() % 4, 0);
        // 32 + (32 + 5 -> 40) + (32 + 7 -> 40) + (32 + 34 -> 68)
        assert_eq!(res.len(), 32 + 40 + 40 + 68);
    }

    #[test]
    fn broken_icon_files_are_refused_with_a_reason() {
        assert!(icon_resource(&[]).is_err());
        assert!(icon_resource(b"not an icon file at all").is_err());
        let mut cut = sample_ico();
        cut.truncate(cut.len() - 3);
        assert!(icon_resource(&cut).unwrap_err().contains("cut off"));
        let mut cursor = sample_ico();
        cursor[2] = 2; // type 2 is a cursor file
        assert!(icon_resource(&cursor).is_err());
        let mut none = sample_ico();
        none[4] = 0;
        assert!(icon_resource(&none).is_err());
    }

    #[test]
    fn the_real_icon_file_is_usable() {
        let path = PathBuf::from(option_env!("CARGO_MANIFEST_DIR").unwrap_or(".")).join("lipi.ico");
        let Ok(ico) = fs::read(&path) else { return }; // nothing to check without the file
        let res = icon_resource(&ico).unwrap();
        let found = read_res(&res);
        let pictures = u16_at(&ico, 4).unwrap() as usize;
        assert_eq!(found.len(), pictures + 2);
        assert!(pictures >= 6, "an exe icon needs the usual sizes");
    }
}