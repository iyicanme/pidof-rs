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
//! # use pidof_rs::ProcessInfoTable;
//!
//!
//! # fn main() -> Result<(), std::io::Error> {
//! let process_info_table =
//!     ProcessInfoTable::populate(None, false, false, false)?;
//!         
//! let process_name = "foo";
//! let pids = process_info_table.pid_of(process_name);
//!
//! dbg!(pids);
//! # Ok(())
//! # }
//! ```

pub use crate::check_flags::{CheckRoot, CheckScripts, CheckThreads, CheckWorkers};
use crate::procfs::Procfs;
use crate::utils::{base_name, pid_link};

mod check_flags;
mod procfs;
mod utils;

/// Holds the information of processes running to be matched against the program name.
pub struct ProcessInfoTable {
    info: Vec<ProcessInfo>,

    check_root: CheckRoot,
    check_scripts: CheckScripts,
    check_workers: CheckWorkers,
}

impl ProcessInfoTable {
    /// Scans the system to gather information of processes.
    /// Table does not refresh, so information can get stale as processes get spawned and die.
    ///
    /// # Arguments
    /// * `check_root` - Discards processes with different root
    /// * `check_scripts` - Also matches names of running scripts
    /// * `check_threads` - Also matches thread names
    /// * `check_workers` - Also matches kernel workers
    ///
    /// # Errors
    /// Returns error if populating the process table fails
    pub fn populate(
        check_root: CheckRoot,
        check_scripts: CheckScripts,
        check_threads: CheckThreads,
        check_workers: CheckWorkers,
    ) -> std::io::Result<Self> {
        let table = Self {
            info: Procfs::new()?.read(check_threads),
            check_root,
            check_scripts,
            check_workers,
        };

        Ok(table)
    }

    /// Scans the table for entries matching the program name and returns list of process IDs
    ///
    /// # Arguments
    /// * `program_name` - Program name to match
    #[must_use]
    pub fn pid_of(&self, program_name: &str) -> Vec<i32> {
        self.info
            .iter()
            .filter(|p| {
                p.matches(
                    program_name,
                    &self.check_root,
                    self.check_workers,
                    self.check_scripts,
                )
            })
            .map(|p| p.tid)
            .collect()
    }
}

#[derive(Debug)]
struct ProcessInfo {
    tid: i32,
    ppid: i32,
    tgid: i32,
    cmd: String,
    cmdline_vector: Vec<String>,
}

impl ProcessInfo {
    #[must_use]
    fn matches(
        &self,
        program_name: &str,
        check_root: &CheckRoot,
        check_workers: CheckWorkers,
        check_scripts: CheckScripts,
    ) -> bool {
        const LOGIN_SHELL_PREFIX: char = '-';

        if let CheckRoot::Yes(pidof_root) = check_root {
            let Ok(link) = pid_link(self.tid, "root") else {
                return false;
            };

            if link.ne(pidof_root) {
                return false;
            }
        }

        let program_base_name = base_name(program_name);
        let mut cmd_line = self
            .cmdline_vector
            .iter()
            .filter(|c| !c.starts_with(LOGIN_SHELL_PREFIX));

        let Some(cmd_arg0) = cmd_line.next() else {
            return false;
        };

        let cmd_arg0_base = base_name(cmd_arg0);
        let Ok(exe_link) = pid_link(self.tid, "exe") else {
            return false;
        };

        let exe_link_base = base_name(&exe_link);

        let condition1 = program_name == cmd_arg0
            || program_name == cmd_arg0_base
            || (check_workers == CheckWorkers::Yes && program_name == self.cmd)
            || program_base_name == cmd_arg0
            || program_name == exe_link
            || program_name == exe_link_base;

        let condition2 = if check_scripts == CheckScripts::Yes {
            cmd_line.next().map_or(false, |cmd_arg1| {
                let cmd_arg1_base = base_name(cmd_arg1);
                self.cmd == cmd_arg1_base
                    || program_name == cmd_arg1_base
                    || program_base_name == cmd_arg1
                    || program_name == cmd_arg1
            })
        } else {
            false
        };

        let condition3 = cmd_arg0.contains('_') && program_name == self.cmd;

        condition1 || condition2 || condition3
    }
}
