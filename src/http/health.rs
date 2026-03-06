use tiny_http::{Request, Response};

pub fn handle(request: Request) {
    let response = Response::from_string("OK").with_status_code(200);
    let _ = request.respond(response);
}
