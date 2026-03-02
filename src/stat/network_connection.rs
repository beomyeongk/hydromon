use crate::db::NetworkConnection;
use procfs::net::{TcpState, tcp, tcp6};
use std::io;

pub struct NetworkConnectionStats;

impl NetworkConnectionStats {
    pub fn new() -> Self {
        NetworkConnectionStats
    }

    pub fn update(&mut self, now_in_secs: i64) -> io::Result<NetworkConnection> {
        // TCP States Aggregation (IPv4 + IPv6)
        let mut tcp_syn_sent = 0;
        let mut tcp_syn_recv = 0;
        let mut tcp_established = 0;
        let mut tcp_time_wait = 0;
        let mut tcp_close_wait = 0;
        let mut tcp_listen = 0;
        let mut tcp_closing = 0;

        let mut process_state = |state: TcpState| match state {
            TcpState::SynSent => tcp_syn_sent += 1,
            TcpState::SynRecv => tcp_syn_recv += 1,
            TcpState::Established => tcp_established += 1,
            TcpState::TimeWait => tcp_time_wait += 1,
            TcpState::CloseWait => tcp_close_wait += 1,
            TcpState::Listen => tcp_listen += 1,
            TcpState::Closing => tcp_closing += 1,
            _ => {}
        };

        // IPv4
        if let Ok(tcp_entries) = tcp() {
            for entry in tcp_entries {
                process_state(entry.state);
            }
        }

        // IPv6
        if let Ok(tcp6_entries) = tcp6() {
            for entry in tcp6_entries {
                process_state(entry.state);
            }
        }

        Ok(NetworkConnection {
            timestamp: now_in_secs,
            tcp_syn_sent,
            tcp_syn_recv,
            tcp_established,
            tcp_time_wait,
            tcp_close_wait,
            tcp_listen,
            tcp_closing,
        })
    }
}
