use crate::http::common::{apply_time_filter, parse_query};
use rusqlite::Connection;
use serde::Serialize;
use tiny_http::{Header, Request, Response};

#[derive(Serialize)]
struct GpuNvidiaRow {
    timestamp: i64,
    name: String,
    fan_speed: u32,
    temp: i32,
    power_w: u32,
    vram_used_mib: u32,
    vram_total_mib: u32,
}

#[derive(Serialize)]
struct GpuNvidiaResponse {
    size: usize,
    data: Vec<GpuNvidiaRow>,
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

    let mut sql =
        "SELECT g.timestamp, n.name, g.fan_speed, g.temp, g.power_w, g.vram_used_mib, g.vram_total_mib \
         FROM gpu_nvidia g JOIN name_map n ON g.name_id = n.id WHERE 1=1"
            .to_string();
    let mut sql_params: Vec<rusqlite::types::Value> = Vec::new();

    apply_time_filter(&params, &mut sql, &mut sql_params);

    sql.push_str(" ORDER BY g.timestamp ASC, n.name ASC");

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
        Ok(GpuNvidiaRow {
            timestamp: row.get(0)?,
            name: row.get(1)?,
            fan_speed: row.get(2)?,
            temp: row.get(3)?,
            power_w: row.get(4)?,
            vram_used_mib: row.get(5)?,
            vram_total_mib: row.get(6)?,
        })
    });

    let mut data = Vec::new();
    if let Ok(iter) = rows_res {
        for row in iter.filter_map(|r| r.ok()) {
            data.push(row);
        }
    }

    let body = GpuNvidiaResponse {
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
