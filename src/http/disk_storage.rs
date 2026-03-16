use crate::http::common::{apply_time_filter, parse_query};
use rusqlite::Connection;
use serde::Serialize;
use tiny_http::{Header, Request, Response};

#[derive(Serialize)]
struct DiskStorageRow {
    timestamp: i64,
    name: String,
    total: u32,    // 1 unit = 16 KiB
    used: u32,     // 1 unit = 16 KiB
    num_inodes: i64,
}

#[derive(Serialize)]
struct DiskStorageResponse {
    size: usize,
    data: Vec<DiskStorageRow>,
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

    let mut sql = "SELECT d.timestamp, n.name, d.total, d.used, d.num_inodes \
                   FROM disk_storage d JOIN name_map n ON d.name_id = n.id WHERE 1=1"
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
        Ok(DiskStorageRow {
            timestamp: row.get(0)?,
            name: row.get(1)?,
            total: row.get(2)?,
            used: row.get(3)?,
            num_inodes: row.get(4)?,
        })
    });

    let mut data = Vec::new();
    if let Ok(iter) = rows_res {
        for row in iter.filter_map(|r| r.ok()) {
            data.push(row);
        }
    }

    let body = DiskStorageResponse {
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
