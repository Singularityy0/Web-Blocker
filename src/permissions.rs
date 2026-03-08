use std::fs::OpenOptions;
use std::path::Path;
pub fn check_permissions(hosts_path: &Path) -> Result<(), String> {
    match OpenOptions::new().append(true).open(hosts_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!(
            "Cannot write to the hosts file ({}). Please re-launch as Administrator.",
            e
        )),
    }
}
