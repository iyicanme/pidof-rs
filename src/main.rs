#![allow(clippy::struct_excessive_bools)]

use std::process::exit;

use clap::Parser;
use nix::libc::{EXIT_FAILURE, EXIT_SUCCESS};

pub use pidof_rs::{CheckRoot, CheckScripts, CheckThreads, CheckWorkers, ProcessInfoTable};

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(short, long = "single-shot", help = "return one PID only")]
    show_single_result: bool,

    #[arg(short, long, help = "omit processes with different root")]
    check_root: bool,

    #[arg(short, help = "quiet mode, only set the exit code")]
    quiet: bool,

    #[arg(short = 'w', long = "with-workers", help = "show kernel workers too")]
    check_workers: bool,

    #[arg(short = 'x', help = "also find shells running the named scripts")]
    check_scripts: bool,

    #[arg(short, long = "omit-pid", help = "omit processes with PID")]
    omitted_pids: Vec<i32>,

    #[arg(short = 't', long = "lightweight", help = "list threads too")]
    check_threads: bool,

    #[arg(
        short = 'S',
        long,
        default_value = " ",
        help = "use SEP as separator put between PIDs"
    )]
    separator: String,

    #[arg(short = 'V', long, help = "output version information and exit")]
    version: bool,

    program_names: Vec<String>,
}

fn main() {
    let args = Args::parse();

    if args.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        exit(EXIT_SUCCESS);
    }

    let check_root = CheckRoot::from(args.check_root);
    let check_scripts = CheckScripts::from(args.check_scripts);
    let check_threads = CheckThreads::from(args.check_threads);
    let check_workers = CheckWorkers::from(args.check_workers);

    let process_info_table =
        ProcessInfoTable::populate(check_root, check_scripts, check_threads, check_workers)
            .expect("process table populated");

    let pids: Vec<i32> = args
        .program_names
        .iter()
        .flat_map(|program| process_info_table.pid_of(program))
        .filter(|pid| !args.omitted_pids.contains(pid))
        .collect();

    let chosen_pids = if args.show_single_result {
        &pids[0..1]
    } else {
        &pids
    };

    if !args.quiet {
        let pid_strings: Vec<String> = chosen_pids
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        let output = pid_strings.join(&args.separator);
        println!("{output}");
    }

    let exit_code = if pids.is_empty() {
        EXIT_FAILURE
    } else {
        EXIT_SUCCESS
    };

    exit(exit_code);
}
