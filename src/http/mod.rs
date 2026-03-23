pub mod common;
pub mod cpu_freqs;
pub mod cpu_modes;
pub mod cpu_usage;
pub mod disk_io;
pub mod disk_storage;
pub mod gpu_nvidia;
pub mod health;
pub mod memory_usage;
pub mod network_connection;
pub mod network_traffic;
pub mod sys_activity;
pub mod sys_summary;
pub mod temperature;

use rusqlite::{Connection, OpenFlags};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use tiny_http::{Method, Server};

pub fn start(addr: &str, db_path: &str, running: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    let server = Server::http(addr).expect("Failed to bind HTTP server");
    let addr = addr.to_string();
    let db_path = db_path.to_string();
    println!("HTTP server listening on {}", addr);

    thread::spawn(move || {
        let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("Failed to open DB for HTTP");

        loop {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            // Non-blocking receive with a short timeout
            match server.recv_timeout(std::time::Duration::from_millis(1000)) {
                Ok(Some(request)) => {
                    let method = request.method().clone();
                    let url = request.url().to_string();
                    let path = url.split('?').next().unwrap_or(url.as_str());

                    match (method, path) {
                        (Method::Get, "/health") => health::handle(request),
                        (Method::Get, "/cpu_modes") => cpu_modes::handle(request, &conn),
                        (Method::Get, "/memory_usage") => memory_usage::handle(request, &conn),
                        (Method::Get, "/cpu_freqs") => cpu_freqs::handle(request, &conn),
                        (Method::Get, "/cpu_usage") => cpu_usage::handle(request, &conn),
                        (Method::Get, "/disk_io") => disk_io::handle(request, &conn),
                        (Method::Get, "/disk_storage") => disk_storage::handle(request, &conn),
                        (Method::Get, "/network_traffic") => network_traffic::handle(request, &conn),
                        (Method::Get, "/network_connection") => network_connection::handle(request, &conn),
                        (Method::Get, "/sys_summary") => sys_summary::handle(request, &conn),
                        (Method::Get, "/sys_activity") => sys_activity::handle(request, &conn),
                        (Method::Get, "/temperature") => temperature::handle(request, &conn),
                        (Method::Get, "/gpu_nvidia") => gpu_nvidia::handle(request, &conn),
                        _ => {
                            let response =
                                tiny_http::Response::from_string("Not Found").with_status_code(404);
                            let _ = request.respond(response);
                        }
                    }
                }
                Ok(None) => {} // timeout — check running flag again
                Err(e) => {
                    eprintln!("HTTP server error: {}", e);
                    break;
                }
            }
        }
        println!("HTTP server stopped.");
    })
}
