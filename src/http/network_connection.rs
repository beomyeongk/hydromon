use crate::http::common::{apply_time_filter, parse_query};
use rusqlite::Connection;
use serde::Serialize;
use tiny_http::{Header, Request, Response};

#[derive(Serialize)]
struct NetworkConnectionRow {
    timestamp: i64,
    tcp_syn_sent: u32,
    tcp_syn_recv: u32,
    tcp_established: u32,
    tcp_time_wait: u32,
    tcp_close_wait: u32,
    tcp_listen: u32,
    tcp_closing: u32,
}

#[derive(Serialize)]
struct NetworkConnectionResponse {
    size: usize,
    data: Vec<NetworkConnectionRow>,
}

pub fn handle(request: Request, conn: &Connection) {
    let params = match parse_query(&request) {
        Ok(p) => p,
        Err(e) => {
            let response = Response::from_string(e).with_status_code(400);
            let _ = request.respond(response);
            return;
        }
    };

    let mut sql = "SELECT timestamp, tcp_syn_sent, tcp_syn_recv, tcp_established, \
                          tcp_time_wait, tcp_close_wait, tcp_listen, tcp_closing \
                   FROM network_connection WHERE 1=1"
        .to_string();
    let mut sql_params: Vec<rusqlite::types::Value> = Vec::new();

    apply_time_filter(&params, &mut sql, &mut sql_params);

    sql.push_str(" ORDER BY timestamp ASC");

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => {
            let response =
                Response::from_string(format!("Database error: {}", e)).with_status_code(500);
            let _ = request.respond(response);
            return;
        }
    };

    let p_refs: Vec<&dyn rusqlite::ToSql> = sql_params
        .iter()
        .map(|v| v as &dyn rusqlite::ToSql)
        .collect();

    let rows_res = stmt.query_map(&*p_refs, |row| {
        Ok(NetworkConnectionRow {
            timestamp: row.get(0)?,
            tcp_syn_sent: row.get(1)?,
            tcp_syn_recv: row.get(2)?,
            tcp_established: row.get(3)?,
            tcp_time_wait: row.get(4)?,
            tcp_close_wait: row.get(5)?,
            tcp_listen: row.get(6)?,
            tcp_closing: row.get(7)?,
        })
    });

    let mut data = Vec::new();
    if let Ok(iter) = rows_res {
        for row in iter.filter_map(|r| r.ok()) {
            data.push(row);
        }
    }

    let body = NetworkConnectionResponse {
        size: data.len(),
        data,
    };

    match serde_json::to_string(&body) {
        Ok(json) => {
            let mut response = Response::from_string(json).with_status_code(200);
            if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]) {
                response.add_header(header);
            }
            let _ = request.respond(response);
        }
        Err(e) => {
            let response =
                Response::from_string(format!("Serialization error: {}", e)).with_status_code(500);
            let _ = request.respond(response);
        }
    }
}
