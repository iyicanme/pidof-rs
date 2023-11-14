use std::fs::read_link;
use std::path::Path;

pub(crate) fn base_name(name: &str) -> &str {
    match name.rsplit_once('/') {
        Some((_, base_name)) => base_name,
        None => name,
    }
}

pub(crate) fn is_root() -> bool {
    nix::unistd::geteuid().as_raw() == 0
}

pub fn pid_link(pid: i32, base_name: &str) -> std::io::Result<String> {
    let link = Path::new("/proc").join(pid.to_string()).join(base_name);

    Ok(read_link(link)?.to_string_lossy().to_string())
}
