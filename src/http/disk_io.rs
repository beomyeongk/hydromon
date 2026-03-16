use crate::http::common::{apply_time_filter, parse_query};
use rusqlite::Connection;
use serde::Serialize;
use tiny_http::{Header, Request, Response};

#[derive(Serialize)]
struct DiskIoRow {
    timestamp: i64,
    name: String,
    r_kbps: u32,
    w_kbps: u32,
    r_await: u32,
    w_await: u32,
    aqu_sz: u32,
    util: u32,
    iops: u32,
}

#[derive(Serialize)]
struct DiskIoResponse {
    size: usize,
    data: Vec<DiskIoRow>,
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

    let mut sql = "SELECT d.timestamp, n.name, d.r_kbps, d.w_kbps, d.r_await, d.w_await, d.aqu_sz, d.util, d.iops \
                   FROM disk_io d JOIN name_map n ON d.name_id = n.id WHERE 1=1"
        .to_string();
    let mut sql_params: Vec<rusqlite::types::Value> = Vec::new();

    apply_time_filter(&params, &mut sql, &mut sql_params);

    sql.push_str(" ORDER BY d.timestamp ASC, n.name ASC");

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
        Ok(DiskIoRow {
            timestamp: row.get(0)?,
            name: row.get(1)?,
            r_kbps: row.get(2)?,
            w_kbps: row.get(3)?,
            r_await: row.get(4)?,
            w_await: row.get(5)?,
            aqu_sz: row.get(6)?,
            util: row.get(7)?,
            iops: row.get(8)?,
        })
    });

    let mut data = Vec::new();
    if let Ok(iter) = rows_res {
        for row in iter.filter_map(|r| r.ok()) {
            data.push(row);
        }
    }

    let body = DiskIoResponse {
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
