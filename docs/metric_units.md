# Metric Units & Encoding

This document describes the unit and encoding of each field as stored in the database, along with the formula to recover the real-world value.

---

## cpu_modes

> Source: `/proc/stat` (total CPU), computed as delta ratio between two samples.

| Field | DB Type | Unit | Recover |
|---|---|---|---|
| `user` | i8 | 1% | `value` % |
| `nice` | i8 | 1% | `value` % |
| `system` | i8 | 1% | `value` % |
| `idle` | i8 | 1% | `value` % |
| `iowait` | i8 | 1% | `value` % |
| `irq` | i8 | 1% | `value` % |
| `softirq` | i8 | 1% | `value` % |
| `steal` | i8 | 1% | `value` % |
| `guest` | i8 | 1% | `value` % |
| `guest_nice` | i8 | 1% | `value` % |

---

## cpu_freqs

> Source: `/proc/cpuinfo`, `cpu MHz` field.

| Field | DB Type | Unit | Recover |
|---|---|---|---|
| `freqs[i]` | i8 (raw binary BLOB) | 100 MHz | `value × 100` MHz |

> BLOB is a packed byte array: each byte stores one `i8` value reinterpreted as `u8` (bitwise identical). Element count = `length(freqs)` bytes.

---

## cpu_usage

> Source: `/proc/stat` (per-core), computed as delta ratio between two samples.

| Field | DB Type | Unit | Recover |
|---|---|---|---|
| `usages[i]` | i8 (raw binary BLOB) | 1% | `value` % |

> BLOB is a packed byte array: each byte stores one `i8` value reinterpreted as `u8` (bitwise identical). Element count = `length(usages)` bytes.

---

## memory_usage

> Source: `/proc/meminfo` (raw unit: KiB).

| Field | DB Type | Unit | Recover |
|---|---|---|---|
| `total` | u32 | 1 MiB | `value` MiB |
| `free` | u32 | 1 MiB | `value` MiB |
| `available` | u32 | 1 MiB | `value` MiB |
| `buffers` | u32 | 1 MiB | `value` MiB |
| `cached` | u32 | 1 MiB | `value` MiB |
| `swap_total` | u32 | 1 MiB | `value` MiB |
| `swap_usage` | u32 | 1 MiB | `value` MiB |

> KiB values from `meminfo` are divided by 1024 before storage.

---

## disk_io

> Source: `/proc/diskstats`, computed as delta per elapsed time between two samples.

| Field | DB Type | Unit | Recover |
|---|---|---|---|
| `r_kbps` | u32 | 1 KiB/s | `value` KiB/s |
| `w_kbps` | u32 | 1 KiB/s | `value` KiB/s |
| `r_await` | u32 | 1 ms | `value` ms |
| `w_await` | u32 | 1 ms | `value` ms |
| `aqu_sz` | u32 | 1/16 | `value / 16` (avg queue depth) |
| `util` | u32 | 1/16 % | `value / 16` % |
| `iops` | u32 | 256 IOPS | `value × 256` IOPS |

> - `r_kbps` = `sectors_read_delta × 512 / 1024 / elapsed_s`
> - `aqu_sz` = `weighted_time_delta / elapsed_ms × 16`
> - `util` = `(time_in_progress_delta / elapsed_ms) × 100 × 16`
> - `iops` = `(reads + writes) / elapsed_s / 256`

---

## disk_storage

> Source: `statvfs()` syscall.

| Field | DB Type | Unit | Recover |
|---|---|---|---|
| `total` | u32 | 16 KiB | `value × 16` KiB |
| `used` | u32 | 16 KiB | `value × 16` KiB |
| `num_inodes` | i64 | 1 inode | `value` inodes (used) |

> `total = (f_blocks × f_bsize) / 16384`. Storing in 16 KiB units allows u32 to represent up to ~64 TiB.

---

## network_traffic

> Source: `/proc/net/dev`, computed as delta per elapsed time between two samples.

| Field | DB Type | Unit | Recover |
|---|---|---|---|
| `rx_kbps` | u32 | 1 KiB/s | `value` KiB/s |
| `tx_kbps` | u32 | 1 KiB/s | `value` KiB/s |
| `rx_pckps` | u32 | 1 pkt/s | `value` packets/s |
| `tx_pckps` | u32 | 1 pkt/s | `value` packets/s |

> `rx_kbps = rx_bytes_delta / 1024 / elapsed_s`

---

## network_connection

> Source: `/proc/net/tcp` + `/proc/net/tcp6` (IPv4 + IPv6 combined).

| Field | DB Type | Unit | Recover |
|---|---|---|---|
| `tcp_syn_sent` | u32 | 1 conn | `value` connections |
| `tcp_syn_recv` | u32 | 1 conn | `value` connections |
| `tcp_established` | u32 | 1 conn | `value` connections |
| `tcp_time_wait` | u32 | 1 conn | `value` connections |
| `tcp_close_wait` | u32 | 1 conn | `value` connections |
| `tcp_listen` | u32 | 1 conn | `value` connections |
| `tcp_closing` | u32 | 1 conn | `value` connections |

---

## sys_summary

> Source: `/proc/uptime`, `/proc/loadavg`, `/proc/sys/fs/file-nr`.

| Field | DB Type | Unit | Recover |
|---|---|---|---|
| `uptime` | u32 | 1 s | `value` seconds |
| `total_tasks` | u32 | 1 task | `value` tasks |
| `load_avg_1m` | u32 | 0.01 | `value / 100` (1-minute load average) |
| `num_fds` | u32 | 1 fd | `value` open file descriptors |

> `load_avg_1m = (float × 100) as u32`

---

## sys_activity

> Source: `/proc/stat` (`intr` / `ctxt` lines), computed as delta per elapsed time between two samples.

| Field | DB Type | Unit | Recover |
|---|---|---|---|
| `intr` | u64 | 1 /s | `value` interrupts/s |
| `ctxt` | u64 | 1 /s | `value` context switches/s |

---

## sys_temp

> Source: `/sys/class/hwmon/hwmon*/temp*_input` (millidegrees Celsius).

| Field | DB Type | Unit | Recover |
|---|---|---|---|
| `temp` | i32 | 1 °C | `value` °C |

> `temp = round(millidegrees / 1000)`

---

## gpu_nvidia

> Source: NVML (NVIDIA Management Library).

| Field | DB Type | Unit | Recover |
|---|---|---|---|
| `fan_speed` | u32 | 1 % | `value` % |
| `temp` | i32 | 1 °C | `value` °C |
| `power_w` | u32 | 1 W | `value` W |
| `vram_used_mib` | u32 | 1 MiB | `value` MiB |
| `vram_total_mib` | u32 | 1 MiB | `value` MiB |
| `gpu_clock_mhz` | u32 | 1 MHz | `value` MHz (current graphics clock) |
| `mem_clock_mhz` | u32 | 1 MHz | `value` MHz (current memory clock) |
| `gpu_util` | u32 | 1 % | `value` % |
| `enc_util` | u32 | 1 % | `value` % (NVML encoder utilization) |
| `dec_util` | u32 | 1 % | `value` % (NVML decoder utilization) |

> `power_w = nvml_power_mW / 1000`  
> `gpu_clock_mhz` and `mem_clock_mhz` are the actual running clocks via `clock_info()`, not the application-set target clocks.  
> `enc_util` / `dec_util` are obtained via `nvmlDeviceGetEncoderUtilization` / `nvmlDeviceGetDecoderUtilization`; supported on Kepler or newer. Returns 0 if unsupported.
