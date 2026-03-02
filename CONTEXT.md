# Project Context & Agent Memory (Hydromon)

> **About this file**: This document is intended to provide rapid context to AI agents (and human developers) working on the `hydromon` project. It summarizes architectural decisions, optimization strategies, and the current state of the codebase.

## 1. Project Overview
**Name**: Hydromon (Hydro Monitor)
**Goal**: A lightweight, high-performance Linux system monitoring daemon written in Rust.
**Philosophy**:
- **Maximize Performance**: Direct system interactions are preferred over heavy abstractions.
- **Minimize Storage**: Every byte counts. Extreme database optimization is applied.
- **Linux-Specific**: Prioritizes Linux `procfs`/`sysfs` features over cross-platform compatibility.

## 2. Technology Stack
- **Language**: Rust (Edition 2021)
- **Database**: SQLite (via `rusqlite` crate)
- **Key Libraries**:
    - `procfs`: `/proc/stat`, `/proc/meminfo`, `/proc/cpuinfo`, et cetra.
    - `serde`, `serde_json`: For JSONB serialization.
    - `ctrlc`: For graceful shutdown handling.
    - `nvml-wrapper`: For direct memory-level C-API interaction with NVIDIA GPUs.

## 3. Key Architectural Decisions (Critical Context)

### A. Data Collection Strategy
- **Why `procfs`?**: Chosen over `sysinfo` as the middle ground between ease of use and raw Linux access with granular control and lower overhead.
- **Manual Parsing**: Some components may still resemble manual logic for speed, but `procfs` structs are used for stability when appropriate.
- **Config Handling**: `hydromon.toml` is the source of truth for device monitoring configuration. If missing, the application automatically detects available physical disks and generates a default configuration file.

### B. Database & Storage Optimization
The database schema (`hydromon.db`) is highly optimized for space efficiency using SQLite:
- **Scaled Integers**: Floating point numbers are aggressively avoided. Metrics (percentages, rates, latencies) are scaled (e.g., x100, x16) and cast to integers (`i8`, `u32`, `u64`) to save space.
- **JSONB Encoding**: Complex parallel metrics (like per-core frequencies) are compacted into SQLite JSONB blob arrays instead of using separate rows.
- **Normalized Tables**: High-cardinality metrics (like disk I/O, network traffic, system temperatures) use normalized tables with composite primary keys (`timestamp`, `device_name`/`interface`/`sensor_label`).

## 4. Current Feature Status
The following metrics and features are currently implemented and actively collected:
- Config Auto-Generation
- CPU Modes & Per-core Frequencies
- Memory & Swap Usage
- Disk I/O & Volume Usage
- Network Traffic
- System Summary (Uptime, Tasks, Load Avg, FDs)
- System Activity (Interrupts, Context Switches)
- System Temperatures (Auto-detected from sysfs)
- GPU Nvidia Metrics (Fan, Temp, Power, VRAM via NVML API)

## 5. Known Quirks & Tips
- **`procfs`**: This version requires importing the `Current` or `CurrentSI` trait to use `KernelStats::current()` or `CpuInfo::current()`.
- **SQLite JSONB**: We use `serde_json` to serialize standard vectors, then rely on SQLite's `jsonb()` function in the `INSERT` query to compress it.

## 6. How to Run
```bash
cargo run
```
*Note: If schema changes occur, the current strategy is often to DROP the table or remove the `.db` file rather than complex migrations.*
