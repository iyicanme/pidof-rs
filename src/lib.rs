//! > **Rust rewrite of the `pidof` Linux command line utility**
//! ## Usage
//!
//! You can use it as a replacement to the original:
//!
//! ```bash
//! $ pidof-rs code
//! 7575 7579 7580 7582 7616 7624 7670 7734 7735 7738
//! ```
//!
//! Or call from your Rust code:
//!
//! ```rust
//! # use pidof_rs::{CheckRoot,CheckScripts, CheckThreads,CheckWorkers,ProcessTable};
//! # fn main() -> Result<(), std::io::Error> {
//! # let check_root = CheckRoot::No;
//! # let check_scripts = CheckScripts::No;
//! # let check_threads = CheckThreads::No;
//! # let check_workers = CheckWorkers::No;
//! let process_info_table =
//!     ProcessTable::populate(check_root, check_scripts, check_threads, check_workers)?;
//!         
//! let process_name = "foo";
//! let pids = process_info_table.pid_of(process_name);
//!
//! dbg!(pids);
//! # Ok(())
//! # }
//! ```

pub use crate::check_flags::{CheckRoot, CheckScripts, CheckThreads, CheckWorkers};
use crate::process::{read_processes, Process};
use std::fs::read_link;
use std::path::Path;

mod check_flags;
mod process;

/// Holds the information of processes running to be matched against the program name.
pub struct ProcessTable {
    info: Vec<Process>,
}

impl ProcessTable {
    /// Scans the system to gather information of processes.
    /// Table does not refresh, so information can get stale as processes get spawned and die.
    ///
    /// # Arguments
    /// * `check_threads` - Also matches thread names
    ///
    /// # Errors
    /// Returns error if populating the process table fails
    pub fn populate(check_threads: CheckThreads) -> std::io::Result<Self> {
        Ok(Self {
            info: read_processes(check_threads),
        })
    }

    /// Scans the table for entries matching the program name and returns list of process IDs
    ///
    /// # Arguments
    /// * `program_name` - Program name to match
    /// * `check_root` - Discards processes with different root
    /// * `check_scripts` - Also matches names of running scripts
    /// * `check_workers` - Also matches kernel workers
    #[must_use]
    pub fn pid_of(
        &self,
        program_name: &str,
        check_root: &CheckRoot,
        check_workers: CheckWorkers,
        check_scripts: CheckScripts,
    ) -> Vec<i32> {
        self.info
            .iter()
            .filter(|p| p.matches(program_name, check_root, check_workers, check_scripts))
            .map(|p| p.tid)
            .collect()
    }
}

fn base_name(name: &str) -> &str {
    match name.rsplit_once('/') {
        Some((_, base_name)) => base_name,
        None => name,
    }
}

fn is_root() -> bool {
    nix::unistd::geteuid().as_raw() == 0
}

fn pid_link(pid: i32, base_name: &str) -> std::io::Result<String> {
    let link = Path::new("/proc").join(pid.to_string()).join(base_name);

    Ok(read_link(link)?.to_string_lossy().to_string())
}
