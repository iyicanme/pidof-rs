use std::fs::{DirEntry, File, ReadDir};
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::check_flags::CheckThreads;
use crate::ProcessInfo;

pub(crate) struct Procfs {
    hide_kernel: bool,
    process_filesystem: ReadDir,
}

const KTHREADD_PID: i32 = 2;

impl Procfs {
    pub(crate) fn new() -> std::io::Result<Self> {
        let table = Self {
            hide_kernel: std::env::var_os("LIBPROC_HIDE_KERNEL").is_some(),
            process_filesystem: std::fs::read_dir(Path::new("/proc"))?,
        };

        Ok(table)
    }

    pub(crate) fn read(self, check_threads: CheckThreads) -> Vec<ProcessInfo> {
        match check_threads {
            CheckThreads::No => self.read_processes(),
            CheckThreads::Yes => self.read_processes_and_tasks(),
        }
    }

    fn read_processes(self) -> Vec<ProcessInfo> {
        self.process_filesystem
            .filter_map(read_process)
            .filter(|p| hide_kernel_thread(self.hide_kernel, p))
            .collect()
    }

    fn read_processes_and_tasks(self) -> Vec<ProcessInfo> {
        if std::fs::read_dir(Path::new("/proc/self/task")).is_err() {
            return Vec::new();
        }

        self.process_filesystem
            .filter_map(read_process)
            .flat_map(read_tasks)
            .filter(|p| hide_kernel_thread(self.hide_kernel, p))
            .collect()
    }
}

const fn hide_kernel_thread(hide_kernel: bool, process_info: &ProcessInfo) -> bool {
    !hide_kernel || !(process_info.ppid == KTHREADD_PID || process_info.tid == KTHREADD_PID)
}

#[allow(clippy::similar_names)]
fn read_process(d: std::io::Result<DirEntry>) -> Option<ProcessInfo> {
    let d = is_ok_and_directory_name_first_letter_nonzero_number(d)?;

    let tid = str::parse(d.file_name().to_str()?).ok()?;
    let tgid = tid;

    let path = d.path();
    let (ppid, cmd) = read_stat_file(&path)?;

    let cmdline_vector = read_cmdline_file(&path);

    if cmdline_vector.is_empty() {
        None
    } else {
        Some(ProcessInfo {
            tid,
            ppid,
            tgid,
            cmd,
            cmdline_vector,
        })
    }
}

fn is_ok_and_directory_name_first_letter_nonzero_number(
    d: std::io::Result<DirEntry>,
) -> Option<DirEntry> {
    let d = d.ok()?;

    let directory_name = d.file_name();
    let directory_name_parsed = directory_name.to_string_lossy();
    match directory_name_parsed.chars().next() {
        Some('1'..='9') => Some(d),
        _ => None,
    }
}

fn read_tasks(p: ProcessInfo) -> Vec<ProcessInfo> {
    let path = Path::new("/proc").join(p.tgid.to_string()).join("task");
    let mut tasks =
        std::fs::read_dir(path).map_or_else(|_| vec![], |r| r.filter_map(read_process).collect());

    tasks.push(p);

    tasks
}

fn read_stat_file(path: &Path) -> Option<(i32, String)> {
    const FIELD_SEPERATOR: &str = " ";
    const PROCESS_NAME_PREFIX: &str = "(";
    const PROCESS_NAME_SUFFIX: &str = ")";

    let file = File::open(path.join("stat")).ok()?;
    let mut buffered = BufReader::new(file);

    let mut content = String::new();
    buffered.read_line(&mut content).ok()?;

    let mut fields = content.split(FIELD_SEPERATOR);

    let process_name = fields
        .nth(1)?
        .strip_prefix(PROCESS_NAME_PREFIX)?
        .strip_suffix(PROCESS_NAME_SUFFIX)?
        .to_owned();
    let ppid = fields.nth(1)?.parse().ok()?;

    Some((ppid, process_name))
}

fn read_cmdline_file(path: &Path) -> Vec<String> {
    let Ok(file) = File::open(path.join("cmdline")) else {
        return Vec::new();
    };

    let buffered = BufReader::new(file);

    buffered.lines().map_while(Result::ok).collect()
}
