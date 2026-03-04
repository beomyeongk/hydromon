pub mod health;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use tiny_http::{Method, Server};

pub fn start(addr: &str, running: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    let server = Server::http(addr).expect("Failed to bind HTTP server");
    let addr = addr.to_string();
    println!("HTTP server listening on {}", addr);

    thread::spawn(move || {
        loop {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            // Non-blocking receive with a short timeout
            match server.recv_timeout(std::time::Duration::from_millis(1000)) {
                Ok(Some(request)) => {
                    let method = request.method().clone();
                    let url = request.url().to_string();

                    match (method, url.as_str()) {
                        (Method::Get, "/health") => health::handle(request),
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
