use std::collections::HashSet;
use std::fs;
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub const MARKER_START: &str = "# Website Blocker - Start";
pub const MARKER_END: &str = "# Website Blocker - End";
pub fn default_hosts_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    } else {
        PathBuf::from("/etc/hosts")
    }
}
pub fn backup(hosts_path: &Path) -> io::Result<()> {
    let backup_path = hosts_path.with_extension("bak");
    fs::copy(hosts_path, &backup_path)?;
    Ok(())
}
pub fn read_content(hosts_path: &Path) -> io::Result<String> {
    fs::read_to_string(hosts_path)
}
pub fn clean(hosts_path: &Path, blocked_domains: &HashSet<String>) -> io::Result<()> {
    let content = read_content(hosts_path)?;

    let kept: Vec<&str> = content
        .lines()
        .filter(|line| {
            let t = line.trim();
            if t.is_empty() || t == MARKER_START || t == MARKER_END {
                return false;
            }
            let mut tokens = t.split_whitespace();
            tokens.next();
            if let Some(hostname) = tokens.next() {
                if blocked_domains.contains(hostname) {
                    return false;
                }
            }
            true
        })
        .collect();

    let mut new_content = kept.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    write_hosts(hosts_path, &new_content)
}
pub fn append_blocks(hosts_path: &Path, entries: &[(String, String)]) -> io::Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let mut content = read_content(hosts_path)?;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push('\n');
    content.push_str(MARKER_START);
    content.push('\n');
    for (v4, v6) in entries {
        content.push_str(v4);
        content.push('\n');
        content.push_str(v6);
        content.push('\n');
    }
    content.push_str(MARKER_END);
    content.push('\n');

    write_hosts(hosts_path, &content)
}
pub fn sanitise_on_startup(hosts_path: &Path) -> io::Result<()> {
    const DNS_MARKER: &str = "# localhost name resolution is handled within DNS itself.";

    let content = read_content(hosts_path)?;

    let start_idx = match content.find(MARKER_START) {
        Some(i) => i,
        None => return Ok(()),
    };
    let end_idx = match content.find(MARKER_END) {
        Some(i) => i + MARKER_END.len(),
        None => return Ok(()),
    };

    let blocker_section = &content[start_idx..end_idx];

    let preserved_end = match content.find(DNS_MARKER) {
        Some(i) => {
            let after = i + DNS_MARKER.len();
            match content.as_bytes().get(after).copied() {
                Some(b'\r') => after + 2,
                _ => after + 1,
            }
        }
        None => start_idx,
    };

    let preserved = &content[..preserved_end];
    let mut new_content = String::with_capacity(preserved.len() + blocker_section.len() + 4);
    new_content.push_str(preserved);
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push('\n');
    new_content.push_str(blocker_section);
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    write_hosts(hosts_path, &new_content)
}

fn write_hosts(hosts_path: &Path, new_content: &str) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    return write_hosts_windows(hosts_path, new_content);

    #[cfg(not(target_os = "windows"))]
    return fs::write(hosts_path, new_content);
}
#[cfg(target_os = "windows")]
fn write_hosts_windows(hosts_path: &Path, new_content: &str) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::FromRawHandle;
    use windows_sys::Win32::Foundation::{GENERIC_WRITE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };

    // Null-terminated UTF-16 path.
    let wide: Vec<u16> = hosts_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect();

    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            0,
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let mut file = unsafe { std::fs::File::from_raw_handle(handle as *mut _) };
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(new_content.as_bytes())?;
    file.flush()?;

    Ok(())
}
