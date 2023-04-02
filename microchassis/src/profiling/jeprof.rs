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
use std::{env, io, num::ParseIntError, process::Command};

#[inline]
pub fn router(sym: &SymbolTable, req: Request<Vec<u8>>) -> http::Result<Response<Vec<u8>>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/pprof/conf") => get_pprof_conf_handler(req),
        (&Method::POST, "/pprof/conf") => post_pprof_conf_handler(req),
        (&Method::GET, "/pprof/heap") => get_pprof_heap_handler(req),
        (&Method::GET, "/pprof/cmdline") => get_pprof_cmdline_handler(req),
        (&Method::GET, "/pprof/symbol") => get_pprof_symbol_handler(sym, req),
        (&Method::POST, "/pprof/symbol") => post_pprof_symbol_handler(sym, req),
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
pub fn get_pprof_symbol_handler(
    sym: &SymbolTable,
    _req: Request<Vec<u8>>,
) -> http::Result<Response<Vec<u8>>> {
    let num_symbols = sym.len();
    let body = format!("num_symbols: {num_symbols}\r\n");
    response_ok(body.into_bytes())
}

/// HTTP handler for POST /pprof/symbol.
#[inline]
pub fn post_pprof_symbol_handler(
    sym: &SymbolTable,
    req: Request<Vec<u8>>,
) -> http::Result<Response<Vec<u8>>> {
    let body = String::from_utf8_lossy(req.body());
    let addrs = body
        .split('+')
        .filter_map(|addr| u64::from_str_radix(addr.trim_start_matches("0x"), 16).ok())
        .map(|addr| (addr, sym.lookup_symbol(addr)))
        .filter_map(|(addr, sym)| sym.map(|(_, sym)| (addr, sym)));

    let mut body = String::new();
    for (addr, sym) in addrs {
        body.push_str(format!("{addr:#x}\t{sym}\r\n").as_str());
    }

    response_ok(body.into_bytes())
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

#[derive(Default, Debug)]
pub struct SymbolTable {
    sym: Vec<(u64, String)>,
}

impl SymbolTable {
    #[inline]
    pub fn load() -> io::Result<Self> {
        let nm_output = run_nm()?;
        let mut sym = SymbolTable::default();
        sym.read_nm(nm_output.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(sym)
    }

    #[inline]
    pub fn read_nm(&mut self, output: &[u8]) -> Result<(), ParseIntError> {
        use std::io::prelude::*;

        let b = io::Cursor::new(output);
        for line in b.lines() {
            let line = line.expect("no I/O, no panic");
            let parts: Vec<_> = line.split_ascii_whitespace().collect();
            if parts.len() < 3 || parts[0] == "U" {
                continue;
            }
            if parts[1] != "t" && parts[1] != "T" {
                continue;
            }

            let address = u64::from_str_radix(parts[0].trim_start_matches("0x"), 16)?;
            let symbol: String = parts[2..].join(" ");
            let symbol = rustc_demangle::demangle(symbol.as_str());

            self.sym.push((address, symbol.to_string()));
        }

        self.sym.sort();

        Ok(())
    }

    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.sym.len()
    }

    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.sym.is_empty()
    }

    #[must_use]
    #[inline]
    pub fn lookup_symbol(&self, addr: u64) -> Option<&(u64, String)> {
        match self.sym.binary_search_by_key(&addr, |(saddr, _)| *saddr) {
            Ok(index) => self.sym.get(index),
            Err(index) => {
                if index == 0 {
                    None
                } else {
                    self.sym.get(index - 1)
                }
            }
        }
    }
}

fn run_nm() -> io::Result<Vec<u8>> {
    let exepath = env::current_exe()?;
    let output =
        Command::new("nm").args(["--numeric-sort", "--no-demangle"]).arg(exepath).output()?;
    Ok(output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symtab_lookup_symbol() {
        let symtab = SymbolTable {
            sym: vec![(123, "Abc".to_string()), (456, "Def".to_string()), (789, "Xyz".to_string())],
        };

        assert_eq!(None, symtab.lookup_symbol(100));
        assert_eq!(Some(&(123, "Abc".to_string())), symtab.lookup_symbol(123));
        assert_eq!(Some(&(123, "Abc".to_string())), symtab.lookup_symbol(200));
        assert_eq!(Some(&(123, "Abc".to_string())), symtab.lookup_symbol(455));
        assert_eq!(Some(&(456, "Def".to_string())), symtab.lookup_symbol(456));
        assert_eq!(Some(&(789, "Xyz".to_string())), symtab.lookup_symbol(800));
    }
}
