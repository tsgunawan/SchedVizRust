# SchedVizRust

SchedVizRust is a native Rust desktop application for simulating and visualizing CPU short-term scheduling algorithms. It supports uniprocessor and multicore scheduling, renders Gantt-style timelines, reports turnaround metrics, and exports results as CSV or 300-DPI PNG images.

## Author

Prof. Ir. Ts. Dr. Teddy Surya Gunawan, International Islamic University Malaysia

## Citation

To cite this code, cite the conference paper:

Teddy Surya Gunawan, "SchedVizRust: Visualizing the Collapse of Scheduling Advantage in MulticoreCPU Scheduling," 12th IEEE International Conference on Instrumentation, Measurement and Application 2026, Kuching, 29-30 July 2026.

## Features

- Uniprocessor scheduling with FCFS, Round Robin, SPN, SRT, and HRRN.
- Multicore scheduling with 2 or 4 cores using FCFS, Round Robin, or SRT.
- Optional multicore on-idle migration to demonstrate load balancing effects.
- Editable process table with built-in examples.
- Synchronized timeline and metrics table scrolling.
- CSV export for per-process metrics.
- PNG export for high-resolution timeline charts.

## Sample Configurations

Uniprocessor configuration:

![Uniprocessor configuration](uniprocessor.png)

Multicore configuration:

![Multicore configuration](multicore.png)

## Requirements

- Rust 1.85.0 or newer. This is the minimum Rust version required for the Rust 2024 edition used by this crate.
- Cargo
- Windows is the primary tested target. The app uses `eframe`/`egui` for the GUI and `rfd` for native file dialogs.

## Compile

Compile the release binary:

```bash
cargo build --release
```

On Windows, the compiled executable is written to:

```text
target\release\schedviz_rust.exe
```

Run tests:

```bash
cargo test --workspace
```

If the project is stored in a cloud-synced folder, use a local target directory to avoid intermittent build-cache issues:

```bash
CARGO_TARGET_DIR=/path/to/local/target cargo run --release
```

## Run

Run directly from Cargo:

```bash
cargo run --release
```

Or run the compiled executable after `cargo build --release`:

```powershell
.\target\release\schedviz_rust.exe
```

## Using The App

1. Edit the process table or load one of the built-in examples.
2. Choose Uniprocessor or Multicore mode.
3. Select the scheduling algorithm or algorithms.
4. Adjust the Round Robin quantum when RR is selected.
5. Click Calculate to update the timeline and metrics.
6. Export CSV or PNG output when needed.

## Scheduling Algorithms

| Algorithm | Type | Notes |
| --- | --- | --- |
| FCFS | Non-preemptive | First-Come, First-Served. |
| RR | Preemptive | Round Robin with quantum 1 through 6. |
| SPN | Non-preemptive | Shortest Process Next. |
| SRT | Preemptive | Shortest Remaining Time. |
| HRRN | Non-preemptive | Highest Response Ratio Next. |

Multicore mode supports FCFS, RR, and SRT. SPN and HRRN are available in uniprocessor mode only.

## References

- [Chapter9Exercises.pdf](Chapter9Exercises.pdf)
- [Chapter9_Scheduling_Solutions.pdf](Chapter9_Scheduling_Solutions.pdf)

## Project Layout

```text
src/main.rs             GUI, rendering, CSV export, PNG export
src/scheduler_core.rs   Scheduling algorithms and metrics
tests/scheduler_tests.rs Core scheduler regression tests
```
