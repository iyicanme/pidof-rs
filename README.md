# pidof-rs

Rust rewrite of the `pidof` Linux command line utility

## Usage

You can use it as a replacement to the original:

```
$ pidof-rs foo
7575 7579 7580 7582 7616 7624 7670 7734 7735 7738
```

Or call from your Rust code:

```
let process_info_table =
    ProcessInfoTable::populate(CheckRoot::No, CheckScripts::No, CheckThreads::No, CheckWorkers::No)?;
         
let process_name = "foo";
let pids = process_info_table.pid_of(process_name).join(" ");

println!("{pids}"); // prints "7575 7579 7580 7582 7616 7624 7670 7734 7735 7738"
```

## Benchmarks

This accidentally outperforms original pidof by 100%

The rewrite:

```
$ hyperfine -w 20 -r 100 'target/release/pidof-rs code'
Benchmark 1: target/release/pidof-rs code
  Time (mean ± σ):      18.2 ms ±   3.8 ms    [User: 2.2 ms, System: 15.7 ms]
  Range (min … max):    10.9 ms …  21.6 ms    100 runs
```

The original:

```
$ hyperfine -w 20 -r 100 'LD_LIBRARY_PATH=library/.libs src/.libs/pidof code'
Benchmark 1: LD_LIBRARY_PATH=library/.libs src/.libs/pidof code  
  Time (mean ± σ):      34.6 ms ±   5.1 ms    [User: 4.8 ms, System: 29.4 ms]
  Range (min … max):    23.2 ms …  39.1 ms    100 runs
```

## Contribution

PRs are welcome to improve code quality, performance, documentation or bugfixes. Licensed under MIT License.
