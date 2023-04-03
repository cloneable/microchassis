// Copyright 2023 Folke Behrens
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Contains HTTP handler for jeprof support (/pprof/heap).
//! Based on <https://gperftools.github.io/gperftools/pprof_remote_servers.html>,
//! <https://jemalloc.net/jemalloc.3.html#mallctl_namespace>,
//! <https://github.com/jemalloc/jemalloc/blob/master/bin/jeprof.in>.

use crate::profiling::mallctl;
use http::{header, Method, Request, Response, StatusCode};
use std::env;

#[inline]
pub fn router(req: Request<Vec<u8>>) -> http::Result<Response<Vec<u8>>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/pprof/conf") => get_pprof_conf_handler(req),
        (&Method::POST, "/pprof/conf") => post_pprof_conf_handler(req),
        (&Method::GET, "/pprof/heap") => get_pprof_heap_handler(req),
        (&Method::GET, "/pprof/cmdline") => get_pprof_cmdline_handler(req),
        (&Method::GET, "/pprof/symbol") => get_pprof_symbol_handler(req),
        (&Method::POST, "/pprof/symbol") => post_pprof_symbol_handler(req),
        (&Method::GET, "/pprof/stats") => get_pprof_stats_handler(req),
        _ => {
            let body = b"Bad Request\r\n";
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .header(header::CONTENT_LENGTH, body.len())
                .body(body.to_vec())
        }
    }
}

#[inline]
pub fn get_pprof_conf_handler(_req: Request<Vec<u8>>) -> http::Result<Response<Vec<u8>>> {
    match mallctl::enabled() {
        Ok(true) => (),
        _ => return response_err("jemalloc profiling not enabled"),
    };

    let Ok(state) = mallctl::active() else {
        return response_err("failed to read prof.active\r\n");
    };
    let Ok(sample) = mallctl::sample_interval() else {
        return response_err("failed to read prof.lg_sample\r\n");
    };
    let body = format!("prof.active:{state},prof.lg_sample:{sample}\r\n");
    response_ok(body.into_bytes())
}

#[inline]
pub fn post_pprof_conf_handler(req: Request<Vec<u8>>) -> http::Result<Response<Vec<u8>>> {
    match mallctl::enabled() {
        Ok(true) => (),
        _ => return response_err("jemalloc profiling not enabled\r\n"),
    };

    let query = parse_malloc_conf_query(req.uri().query());

    for (name, value) in query {
        if let Err(e) = match name {
            "prof.reset" => {
                let Some(sample) = value.map(|v| v.parse().ok()) else {
                    return response_err(format!("invalid prof.reset value: {value:?}\r\n").as_str());
                };
                mallctl::reset(sample)
            }
            "prof.active" => {
                let Some(value) = value else {
                    return response_err("prof.active needs value\r\n");
                };
                let Some(state) = value.parse().ok() else {
                    return response_err(format!("invalid prof.active value: {value:?}\r\n").as_str());
                };
                mallctl::set_active(state)
            }
            _ => {
                return response_err(format!("{name}={value:?} unknown\r\n").as_str());
            }
        } {
            return response_err(format!("{name}={value:?} failed: {e}\r\n").as_str());
        }
    }

    response_ok(b"OK\r\n".to_vec())
}

#[inline]
pub fn get_pprof_heap_handler(_req: Request<Vec<u8>>) -> http::Result<Response<Vec<u8>>> {
    match mallctl::enabled() {
        Ok(true) => (),
        _ => return response_err("jemalloc profiling not enabled\r\n"),
    };

    let Ok(f) = tempfile::Builder::new().prefix("jemalloc.").suffix(".prof").tempfile() else {
        return response_err("cannot create temporary file for profile dump\r\n");
    };

    let Ok(profile) = mallctl::dump(f.path().to_str()) else {
        return response_err("failed to dump profile\r\n");
    };

    let filename = f.path().file_name().expect("proper filename from tempfile");
    response_ok_binary(profile.expect("profile not None"), filename.to_string_lossy().as_ref())
}

/// HTTP handler for GET /pprof/cmdline.
#[inline]
pub fn get_pprof_cmdline_handler(_req: Request<Vec<u8>>) -> http::Result<Response<Vec<u8>>> {
    let mut body = String::new();
    for arg in env::args() {
        body.push_str(arg.as_str());
        body.push_str("\r\n");
    }
    response_ok(body.into_bytes())
}

/// HTTP handler for GET /pprof/symbol.
#[inline]
pub fn get_pprof_symbol_handler(_req: Request<Vec<u8>>) -> http::Result<Response<Vec<u8>>> {
    // TODO: any quick way to check if binary is stripped?
    let body = b"num_symbols: 1\r\n";
    response_ok(body.to_vec())
}

/// HTTP handler for POST /pprof/symbol.
#[inline]
pub fn post_pprof_symbol_handler(req: Request<Vec<u8>>) -> http::Result<Response<Vec<u8>>> {
    fn lookup_symbol(addr: u64) -> Option<String> {
        let mut s: Option<String> = None;
        backtrace::resolve(addr as *mut _, |symbol| {
            s = symbol.name().map(|n| n.to_string());
        });
        s
    }

    let body = String::from_utf8_lossy(req.body());
    let addrs = body
        .split('+')
        .filter_map(|addr| u64::from_str_radix(addr.trim_start_matches("0x"), 16).ok())
        .map(|addr| (addr, lookup_symbol(addr)))
        .filter_map(|(addr, sym)| sym.map(|sym| (addr, sym)));

    let mut body = String::new();
    for (addr, sym) in addrs {
        body.push_str(format!("{addr:#x}\t{sym}\r\n").as_str());
    }

    response_ok(body.into_bytes())
}

/// HTTP handler for GET /pprof/stats.
#[inline]
pub fn get_pprof_stats_handler(_req: Request<Vec<u8>>) -> http::Result<Response<Vec<u8>>> {
    let body = match mallctl::stats() {
        Ok(body) => body,
        Err(e) => return response_err(format!("failed to print stats: {e}\r\n").as_str()),
    };
    response_ok(body)
}

fn parse_malloc_conf_query(query: Option<&str>) -> Vec<(&str, Option<&str>)> {
    query
        .map(|q| {
            q.split(',')
                .map(|kv| kv.splitn(2, ':').collect::<Vec<_>>())
                .map(|v| match v.len() {
                    1 => (v[0], None),
                    2 => (v[0], Some(v[1])),
                    _ => unreachable!(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn response_ok(body: Vec<u8>) -> http::Result<Response<Vec<u8>>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=UTF-8")
        .header(header::CONTENT_LENGTH, body.len())
        .body(body)
}

fn response_ok_binary(body: Vec<u8>, filename: &str) -> http::Result<Response<Vec<u8>>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{filename}\""))
        .header(header::CONTENT_LENGTH, body.len())
        .body(body)
}

fn response_err(msg: &str) -> http::Result<Response<Vec<u8>>> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(header::CONTENT_TYPE, "text/plain; charset=UTF-8")
        .header(header::CONTENT_LENGTH, msg.len())
        .body(msg.as_bytes().to_owned())
}
