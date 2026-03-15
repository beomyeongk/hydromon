use crate::http::common::{apply_time_filter, parse_query};
use rusqlite::Connection;
use serde::Serialize;
use tiny_http::{Header, Request, Response};

#[derive(Serialize)]
struct SysTempRow {
    timestamp: i64,
    device: String,  // name_map 조인 (device_id → name)
    sensor: String,  // name_map 조인 (sensor_id → name)
    temp: i32,
}

#[derive(Serialize)]
struct SysTempResponse {
    size: usize,
    data: Vec<SysTempRow>,
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

    // device_id, sensor_id 각각 name_map에 JOIN
    let mut sql =
        "SELECT t.timestamp, nd.name AS device, ns.name AS sensor, t.temp \
         FROM sys_temp t \
         JOIN name_map nd ON t.device_id = nd.id \
         JOIN name_map ns ON t.sensor_id = ns.id \
         WHERE 1=1"
            .to_string();
    let mut sql_params: Vec<rusqlite::types::Value> = Vec::new();

    apply_time_filter(&params, &mut sql, &mut sql_params);

    sql.push_str(" ORDER BY t.timestamp ASC, nd.name ASC, ns.name ASC");

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
        Ok(SysTempRow {
            timestamp: row.get(0)?,
            device: row.get(1)?,
            sensor: row.get(2)?,
            temp: row.get(3)?,
        })
    });

    let mut data = Vec::new();
    if let Ok(iter) = rows_res {
        for row in iter.filter_map(|r| r.ok()) {
            data.push(row);
        }
    }

    let body = SysTempResponse {
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
