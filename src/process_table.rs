use std::fs::{DirEntry, File, ReadDir};
use std::io::{BufRead, BufReader};

use crate::check_flags::CheckThreads;
use crate::ProcessInfo;

pub(crate) struct SlashProc {
    hide_kernel: bool,
    process_filesystem: ReadDir,
}

impl SlashProc {
    pub(crate) fn new() -> std::io::Result<Self> {
        let table = Self {
            hide_kernel: std::env::var_os("LIBPROC_HIDE_KERNEL").is_some(),
            process_filesystem: std::fs::read_dir("/proc")?,
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
            .filter(|p: &ProcessInfo| !self.hide_kernel || !(p.ppid == 2 || p.tid == 2))
            .collect()
    }

    fn read_processes_and_tasks(self) -> Vec<ProcessInfo> {
        if std::fs::read_dir("/proc/self/task").is_err() {
            return vec![];
        }

        self.process_filesystem
            .filter_map(read_process)
            .flat_map(read_tasks)
            .filter(|p: &ProcessInfo| !self.hide_kernel || !(p.ppid == 2 || p.tid == 2))
            .collect()
    }
}

fn read_process(d: std::io::Result<DirEntry>) -> Option<ProcessInfo> {
    let d = is_ok_and_directory_name_first_letter_nonzero_number(d)?;

    let file_name = d.file_name();
    let file_name_string = file_name.to_str()?;

    let tid = str::parse(file_name_string).ok()?;
    let tgid = tid;

    let path = d.path().to_str()?.to_owned();
    let (ppid, cmd) = read_stat_file(&path)?;

    let cmdline_vector = read_cmdline_file(&path)?;

    let process_info = ProcessInfo {
        tid,
        ppid,
        tgid,
        cmd,
        cmdline_vector,
    };

    Some(process_info)
}

fn is_ok_and_directory_name_first_letter_nonzero_number(
    d: std::io::Result<DirEntry>,
) -> Option<DirEntry> {
    let d = d.ok()?;

    let directory_name = d.file_name();
    let directory_name_parsed = directory_name.to_string_lossy();
    match directory_name_parsed.chars().next() {
        Some('1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9') => Some(d),
        _ => None,
    }
}

fn read_tasks(p: ProcessInfo) -> Vec<ProcessInfo> {
    let mut tasks = if let Ok(r) = std::fs::read_dir(format!("/proc/{}/task", p.tgid)) {
        r.filter_map(read_process).collect()
    } else {
        vec![]
    };

    tasks.push(p);

    tasks
}

fn read_stat_file(path: &str) -> Option<(i32, String)> {
    const FIELD_SEPERATOR: &str = " ";
    const PROCESS_NAME_PREFIX: &str = "(";
    const PROCESS_NAME_SUFFIX: &str = ")";

    let file = File::open(format!("{path}/stat")).ok()?;
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

fn read_cmdline_file(path: &str) -> Option<Vec<String>> {
    let file = File::open(format!("{path}/cmdline")).ok()?;
    let mut buffered = BufReader::new(file);

    let mut lines = vec![];

    let mut content = String::new();
    while let Ok(amount_read) = buffered.read_line(&mut content) {
        if amount_read == 0 {
            break;
        }

        lines.push(content.clone());
    }

    Some(lines)
}
