pub(crate) fn base_name(name: &str) -> &str {
    match name.rsplit_once('/') {
        Some((_, base_name)) => base_name,
        None => name,
    }
}

pub(crate) fn is_root() -> bool {
    nix::unistd::geteuid().as_raw() == 0
}

pub fn pid_link(pid: i32, base_name: &str) -> Result<String, nix::errno::Errno> {
    let link = format!("/proc/{pid}/{base_name}");

    Ok(nix::fcntl::readlink(link.as_str())?
        .to_string_lossy()
        .to_string())
}
