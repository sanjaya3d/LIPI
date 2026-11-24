// Run on startup: one value in the current user's Run list in the registry.
//
//   HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
//       LIPI = "C:\path\to\lipi.exe"
//
// It needs no administrator rights and affects only this Windows user. Turning it off deletes
// the value. Whether the window opens or hides in the tray at that start is the separate
// start_in_tray setting.
//
// Task Manager's Startup tab can turn an entry off without touching the Run value. It keeps
// that choice in StartupApproved\Run (first byte odd means off), so that is checked too, and
// turning the switch on again clears it.

use std::path::Path;

/// The name of the value in the Run list.
pub const VALUE_NAME: &str = "LIPI";

/// What is stored in the Run value: the full path of the program, in quotes so a path with
/// spaces works.
pub fn command_line(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

/// True if the stored text starts this same program. Case, quotes and spaces around it do not matter.
pub fn same_command(stored: &str, wanted: &str) -> bool {
    fn plain(text: &str) -> String {
        text.trim().trim_matches('"').trim().to_ascii_lowercase()
    }
    let stored = plain(stored);
    !stored.is_empty() && stored == plain(wanted)
}

/// The StartupApproved value is 12 bytes. When the first byte is odd (03 or 07), the entry
/// was turned off in Task Manager. 02 or 06 mean on.
pub fn approval_is_off(data: &[u8]) -> bool {
    data.first().map_or(false, |first| first & 1 == 1)
}

#[cfg(windows)]
pub use windows_registry::{is_enabled, set_enabled};

#[cfg(windows)]
mod windows_registry {
    use super::{approval_is_off, command_line, same_command, VALUE_NAME};

    const HKEY_CURRENT_USER: isize = 0x8000_0001_u32 as i32 as isize;
    const KEY_QUERY_VALUE: u32 = 0x0001;
    const KEY_SET_VALUE: u32 = 0x0002;
    const REG_SZ: u32 = 1;
    const REG_EXPAND_SZ: u32 = 2;
    const ERROR_FILE_NOT_FOUND: i32 = 2;

    const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    const APPROVED_KEY: &str =
        "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(key: isize, sub_key: *const u16, options: u32, access: u32, result: *mut isize) -> i32;
        fn RegQueryValueExW(
            key: isize,
            name: *const u16,
            reserved: *const u32,
            kind: *mut u32,
            data: *mut u8,
            size: *mut u32,
        ) -> i32;
        fn RegSetValueExW(key: isize, name: *const u16, reserved: u32, kind: u32, data: *const u8, size: u32) -> i32;
        fn RegDeleteValueW(key: isize, name: *const u16) -> i32;
        fn RegCloseKey(key: isize) -> i32;
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Opens a key of the current user, runs `job` on it, and closes it again.
    fn with_key<T>(sub_key: &str, access: u32, job: impl FnOnce(isize) -> T) -> Result<T, i32> {
        let mut key: isize = 0;
        let status = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, wide(sub_key).as_ptr(), 0, access, &mut key) };
        if status != 0 {
            return Err(status);
        }
        let result = job(key);
        unsafe { RegCloseKey(key) };
        Ok(result)
    }

    /// Reads one value: its type and its bytes. None if the key or the value is not there.
    fn read_value(sub_key: &str, name: &str) -> Option<(u32, Vec<u8>)> {
        with_key(sub_key, KEY_QUERY_VALUE, |key| {
            let name = wide(name);
            let mut kind = 0u32;
            let mut size = 0u32;
            // First ask how big it is, then read it.
            let status = unsafe { RegQueryValueExW(key, name.as_ptr(), std::ptr::null(), &mut kind, std::ptr::null_mut(), &mut size) };
            if status != 0 || size == 0 || size > 64 * 1024 {
                return None;
            }
            let mut data = vec![0u8; size as usize];
            let status = unsafe { RegQueryValueExW(key, name.as_ptr(), std::ptr::null(), &mut kind, data.as_mut_ptr(), &mut size) };
            if status != 0 {
                return None;
            }
            data.truncate(size as usize);
            Some((kind, data))
        })
        .ok()
        .flatten()
    }

    fn text_of(data: &[u8]) -> String {
        let units: Vec<u16> = data.chunks_exact(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect();
        String::from_utf16_lossy(&units).trim_end_matches('\0').to_string()
    }

    fn write_text(sub_key: &str, name: &str, value: &str) -> Result<(), i32> {
        let bytes: Vec<u8> = wide(value).iter().flat_map(|unit| unit.to_le_bytes()).collect();
        let name = wide(name);
        let status = with_key(sub_key, KEY_SET_VALUE, |key| unsafe {
            RegSetValueExW(key, name.as_ptr(), 0, REG_SZ, bytes.as_ptr(), bytes.len() as u32)
        })?;
        if status == 0 { Ok(()) } else { Err(status) }
    }

    /// Deletes a value. A value that was not there is fine.
    fn delete_value(sub_key: &str, name: &str) -> Result<(), i32> {
        let name = wide(name);
        let status = match with_key(sub_key, KEY_SET_VALUE, |key| unsafe { RegDeleteValueW(key, name.as_ptr()) }) {
            Ok(status) => status,
            Err(ERROR_FILE_NOT_FOUND) => return Ok(()),
            Err(other) => return Err(other),
        };
        if status == 0 || status == ERROR_FILE_NOT_FOUND { Ok(()) } else { Err(status) }
    }

    fn this_command() -> Result<String, String> {
        std::env::current_exe()
            .map(|exe| command_line(&exe))
            .map_err(|e| format!("could not find the program's own path: {e}"))
    }

    /// True if Windows will start this exact program when the user signs in.
    pub fn is_enabled() -> bool {
        let Ok(wanted) = this_command() else { return false };
        let Some((kind, data)) = read_value(RUN_KEY, VALUE_NAME) else { return false };
        if kind != REG_SZ && kind != REG_EXPAND_SZ {
            return false;
        }
        if !same_command(&text_of(&data), &wanted) {
            return false;
        }
        // Turned off in Task Manager?
        !matches!(read_value(APPROVED_KEY, VALUE_NAME), Some((_, flags)) if approval_is_off(&flags))
    }

    /// Turns run on startup on or off. The error is a short sentence for the log.
    pub fn set_enabled(on: bool) -> Result<(), String> {
        if on {
            let command = this_command()?;
            write_text(RUN_KEY, VALUE_NAME, &command).map_err(|code| format!("Windows error {code} while writing the Run list"))?;
            // Clear an old "off" mark from Task Manager. Not being able to is not a problem.
            let _ = delete_value(APPROVED_KEY, VALUE_NAME);
            Ok(())
        } else {
            delete_value(RUN_KEY, VALUE_NAME).map_err(|code| format!("Windows error {code} while changing the Run list"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stored_command_has_the_path_in_quotes() {
        let cmd = command_line(Path::new("C:\\Program Files\\LIPI\\lipi.exe"));
        assert_eq!(cmd, "\"C:\\Program Files\\LIPI\\lipi.exe\"");
    }

    #[test]
    fn the_same_program_is_recognised_in_a_different_spelling() {
        let wanted = command_line(Path::new("C:\\Tools\\lipi.exe"));
        assert!(same_command("\"C:\\Tools\\lipi.exe\"", &wanted));
        assert!(same_command("C:\\TOOLS\\LIPI.EXE", &wanted));
        assert!(same_command("  \"c:\\tools\\lipi.exe\"  ", &wanted));
    }

    #[test]
    fn another_program_or_an_empty_value_is_not_us() {
        let wanted = command_line(Path::new("C:\\Tools\\lipi.exe"));
        assert!(!same_command("\"C:\\Old\\lipi.exe\"", &wanted));
        assert!(!same_command("", &wanted));
        assert!(!same_command("\"\"", &wanted));
        assert!(!same_command("\"C:\\Tools\\lipi.exe\" --something", &wanted));
    }

    #[test]
    fn task_manager_marks_are_read_correctly() {
        // 02 and 06 mean enabled, 03 and 07 mean disabled, the rest of the 12 bytes is a time
        assert!(!approval_is_off(&[0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]));
        assert!(!approval_is_off(&[0x06, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]));
        assert!(approval_is_off(&[0x03, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8]));
        assert!(approval_is_off(&[0x07, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]));
        assert!(!approval_is_off(&[]));
    }
}
