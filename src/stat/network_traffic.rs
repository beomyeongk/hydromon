use crate::db::NetworkTraffic;
use procfs::net::dev_status;
use std::collections::HashMap;
use std::io;
use std::time::Instant;

struct PrevNetStat {
    rx_bytes: u64,
    tx_bytes: u64,
    rx_packets: u64,
    tx_packets: u64,
    timestamp: Instant,
}

pub struct NetworkTrafficStats {
    prev_stats: HashMap<String, PrevNetStat>,
}

impl NetworkTrafficStats {
    pub fn new() -> Self {
        NetworkTrafficStats {
            prev_stats: HashMap::new(),
        }
    }

    pub fn update(
        &mut self,
        now_in_secs: i64,
        target_interfaces: &[String],
    ) -> io::Result<Vec<NetworkTraffic>> {
        let current_net_dev = dev_status().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let now = Instant::now();
        let mut results = Vec::new();

        for (interface_name, stat) in current_net_dev {
            if !target_interfaces.contains(&interface_name) {
                continue;
            }

            if let Some(prev) = self.prev_stats.get(&interface_name) {
                let time_delta = now.duration_since(prev.timestamp).as_secs_f64();

                if time_delta > 0.0 {
                    let rx_bytes_delta = stat.recv_bytes.saturating_sub(prev.rx_bytes);
                    let tx_bytes_delta = stat.sent_bytes.saturating_sub(prev.tx_bytes);
                    let rx_packets_delta = stat.recv_packets.saturating_sub(prev.rx_packets);
                    let tx_packets_delta = stat.sent_packets.saturating_sub(prev.tx_packets);

                    let rx_kbps = (rx_bytes_delta as f64 / 1024.0 / time_delta) as u32;
                    let tx_kbps = (tx_bytes_delta as f64 / 1024.0 / time_delta) as u32;
                    let rx_pckps = (rx_packets_delta as f64 / time_delta) as u32;
                    let tx_pckps = (tx_packets_delta as f64 / time_delta) as u32;

                    results.push(NetworkTraffic {
                        timestamp: now_in_secs,
                        interface: interface_name.clone(),
                        rx_kbps,
                        tx_kbps,
                        rx_pckps,
                        tx_pckps,
                    });
                }
            }

            self.prev_stats.insert(
                interface_name,
                PrevNetStat {
                    rx_bytes: stat.recv_bytes,
                    tx_bytes: stat.sent_bytes,
                    rx_packets: stat.recv_packets,
                    tx_packets: stat.sent_packets,
                    timestamp: now,
                },
            );
        }

        Ok(results)
    }
}
