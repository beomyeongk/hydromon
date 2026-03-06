use std::collections::HashMap;
use tiny_http::Request;

pub struct QueryParams {
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    #[allow(dead_code)]
    pub interval: Option<String>,
}

pub fn parse_query(request: &Request) -> Result<QueryParams, String> {
    let url = request.url().to_string();
    let query_str = url.splitn(2, '?').nth(1).unwrap_or("");

    let mut params = HashMap::new();
    for pair in query_str.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut kv = pair.splitn(2, '=');
        let k = kv.next().unwrap_or("");
        let v = kv.next().unwrap_or("");
        params.insert(k.to_string(), v.to_string());
    }

    // Interval validation
    let valid_intervals = ["30s", "5m", "15m", "1h", "6h", "1d"];
    let interval = if let Some(int) = params.get("interval") {
        if !valid_intervals.contains(&int.as_str()) {
            return Err("Invalid interval. Must be one of: 30s, 5m, 15m, 1h, 6h, 1d".to_string());
        }
        Some(int.to_string())
    } else {
        None
    };

    let start_date = params.get("start_date").and_then(|s| s.parse::<i64>().ok());
    let end_date = params.get("end_date").and_then(|s| s.parse::<i64>().ok());

    Ok(QueryParams {
        start_date,
        end_date,
        interval,
    })
}

pub fn apply_time_filter(
    params: &QueryParams,
    sql: &mut String,
    sql_params: &mut Vec<rusqlite::types::Value>,
) {
    if let Some(start) = params.start_date {
        sql.push_str(" AND timestamp >= ?");
        sql_params.push(rusqlite::types::Value::Integer(start));
    }

    if let Some(end) = params.end_date {
        sql.push_str(" AND timestamp <= ?");
        sql_params.push(rusqlite::types::Value::Integer(end));
    }
}
