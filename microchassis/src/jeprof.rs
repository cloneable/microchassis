//! Contains HTTP handler for jeprof support (/pprof/heap).
//! Based on <https://gperftools.github.io/gperftools/pprof_remote_servers.html>.
//! <https://jemalloc.net/jemalloc.3.html#mallctl_namespace>
//! <https://github.com/jemalloc/jemalloc/blob/master/bin/jeprof.in>

use http::{header, Request, Response, StatusCode};
use std::env;

#[inline]
pub fn get_pprof_heap_handler(_req: Request<()>) -> http::Result<Response<Vec<u8>>> {
    // TODO: impl
    let body = String::new();
    response_ok(body.into_bytes())
}

/// HTTP handler for GET /pprof/cmdline.
#[inline]
pub fn get_pprof_cmdline_handler(_req: Request<()>) -> http::Result<Response<Vec<u8>>> {
    let mut body = String::new();
    for arg in env::args() {
        body.push_str(arg.as_str());
        body.push_str("\r\n");
    }
    response_ok(body.into_bytes())
}

/// HTTP handler for GET /pprof/symbol.
#[inline]
pub fn get_pprof_symbol_handler(_req: Request<()>) -> http::Result<Response<Vec<u8>>> {
    let num_symbols = 0;
    // TODO: impl
    let body = format!("num_symbols: {num_symbols}\r\n");
    response_ok(body.into_bytes())
}

/// HTTP handler for POST /pprof/symbol.
#[inline]
pub fn post_pprof_symbol_handler(_req: Request<Vec<u8>>) -> http::Result<Response<Vec<u8>>> {
    // TODO: impl
    let body = Vec::new();
    response_ok(body)
}

fn response_ok(body: Vec<u8>) -> http::Result<Response<Vec<u8>>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, body.len())
        .body(body)
}
