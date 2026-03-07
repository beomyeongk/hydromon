use crate::db::CpuFreqs;
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

    // Use json() to ensure the jsonb BLOB is returned as a JSON text string
    let mut sql = "SELECT timestamp, json(freqs) FROM cpu_freqs WHERE 1=1".to_string();
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
        let freqs_str: String = row.get(1)?;
        let freqs: Vec<i8> = serde_json::from_str(&freqs_str).unwrap_or_default();
        Ok(CpuFreqs {
            timestamp: row.get(0)?,
            freqs,
        })
    });

    let mut usages = Vec::new();
    if let Ok(iter) = rows_res {
        for row in iter.filter_map(|r| r.ok()) {
            usages.push(row);
        }
    }

    match serde_json::to_string(&usages) {
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
