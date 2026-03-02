# hydromon

A lightweight system monitoring daemon written in Rust.

## Features

- **CPU**: Usage, frequencies, and CPU states (user, system, idle, iowait, etc.)
- **Memory**: RAM and Swap usage
- **Disk**: I/O throughput/latency and storage space
- **Network**: Traiffic and connection states
- **System**: Load average, uptime, temperatures, and interrupts
- **GPU**: NVIDIA GPU metrics (fan speed, temp, power, VRAM) via NVML

## Quick Start

1. **Build and run**
   ```bash
   cargo build --release
   ./target/release/hydromon
   ```

2. **Configuration**
   On the first run, it will automatically generate a `hydromon.toml` config file by detecting your current hardware (disks, network interfaces, temp sensors, etc.). You can edit this file to enable or disable specific metrics. (Check out `hydromon.toml.template` for an example).

3. **Data**
   Metrics are stored in `hydromon.db` (SQLite) in the same directory. You can easily query the data:
   ```bash
   sqlite3 hydromon.db "SELECT * FROM sys_summary ORDER BY timestamp DESC LIMIT 5;"
   ```

## License

See the `LICENSE` file for details.
