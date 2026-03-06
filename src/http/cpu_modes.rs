use crate::db::CpuModes;
use crate::http::common::{apply_time_filter, parse_query};
use rusqlite::Connection;
use tiny_http::{Header, Request, Response};

pub fn handle(request: Request, conn: &Connection) {
    let params = match parse_query(&request) {
        Ok(p) => p,
        Err(e) => {
            let response = Response::from_string(e).with_status_code(400);
            let _ = request.respond(response);
            return;
        }
    };

    let mut sql = "SELECT timestamp, user, nice, system, idle, iowait, irq, softirq, steal, guest, guest_nice FROM cpu_modes WHERE 1=1".to_string();
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
        Ok(CpuModes {
            timestamp: row.get(0)?,
            user: row.get(1)?,
            nice: row.get(2)?,
            system: row.get(3)?,
            idle: row.get(4)?,
            iowait: row.get(5)?,
            irq: row.get(6)?,
            softirq: row.get(7)?,
            steal: row.get(8)?,
            guest: row.get(9)?,
            guest_nice: row.get(10)?,
        })
    });

    let mut modes = Vec::new();
    if let Ok(iter) = rows_res {
        for row in iter.filter_map(|r| r.ok()) {
            modes.push(row);
        }
    }

    match serde_json::to_string(&modes) {
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
