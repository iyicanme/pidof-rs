use crate::check_flags::CheckThreads;
use crate::{base_name, pid_link};
use crate::{CheckRoot, CheckScripts, CheckWorkers};

#[derive(Debug)]
pub struct Process {
    pub(crate) tid: i32,
    pub(crate) ppid: i32,
    pub(crate) cmd: String,
    pub(crate) cmdline_vector: Vec<String>,
}

impl Process {
    #[must_use]
    pub fn matches(
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

pub fn read_processes(check_threads: CheckThreads) -> Vec<Process> {
    procfs::process::all_processes().map_or_else(
        |_| Vec::new(),
        |processes| {
            processes
                .flat_map(|p| read_process(p, check_threads))
                .filter(hide_kernel_thread)
                .collect()
        },
    )
}

fn read_process(
    p: procfs::ProcResult<procfs::process::Process>,
    check_threads: CheckThreads,
) -> Vec<Process> {
    let Ok(process) = p else {
        return Vec::new();
    };

    let mut processes = if check_threads == CheckThreads::Yes {
        process
            .tasks()
            .map_or_else(|_| Vec::new(), |t| t.filter_map(read_thread).collect())
    } else {
        Vec::new()
    };

    if let (Ok(stat), Ok(cmdline)) = (process.stat(), process.cmdline()) {
        processes.push(Process {
            tid: stat.pid,
            ppid: stat.ppid,
            cmd: stat.comm,
            cmdline_vector: cmdline,
        });
    };

    processes
}

fn read_thread(t: procfs::ProcResult<procfs::process::Task>) -> Option<Process> {
    let task = t.ok()?;
    let stat = task.stat().ok()?;

    Some(Process {
        tid: stat.pid,
        ppid: stat.ppid,
        cmd: stat.comm,
        cmdline_vector: Vec::new(),
    })
}

fn hide_kernel_thread(process_info: &Process) -> bool {
    const KTHREADD_PID: i32 = 2;
    let hide_kernel = std::env::var_os("LIBPROC_HIDE_KERNEL").is_some();

    !hide_kernel || !(process_info.ppid == KTHREADD_PID || process_info.tid == KTHREADD_PID)
}
