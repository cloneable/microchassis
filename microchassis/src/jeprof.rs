//! Contains HTTP handler for jeprof support (/pprof/heap).
//! Based on <https://gperftools.github.io/gperftools/pprof_remote_servers.html>,
//! <https://jemalloc.net/jemalloc.3.html#mallctl_namespace>,
//! <https://github.com/jemalloc/jemalloc/blob/master/bin/jeprof.in>.

use http::{header, Method, Request, Response, StatusCode};
use std::{env, io, num::ParseIntError};

#[inline]
pub fn router(sym: &SymbolTable, req: Request<Vec<u8>>) -> http::Result<Response<Vec<u8>>> {
    match (req.method(), req.uri().path()) {
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
pub fn get_pprof_heap_handler(_req: Request<Vec<u8>>) -> http::Result<Response<Vec<u8>>> {
    // TODO: impl
    let body = String::new();
    response_ok(body.into_bytes())
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
        .filter_map(|addr| addr.parse().ok())
        .map(|addr| (addr, sym.lookup_symbol(addr)))
        .filter_map(|(addr, sym)| sym.map(|(_, sym)| (addr, sym)));

    let mut body = String::new();
    for (addr, sym) in addrs {
        body.push_str(format!("{addr:0x}\t{sym}\r\n").as_str());
    }

    response_ok(body.into_bytes())
}

fn response_ok(body: Vec<u8>) -> http::Result<Response<Vec<u8>>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, body.len())
        .body(body)
}

#[derive(Default, Debug)]
pub struct SymbolTable {
    sym: Vec<(u64, String)>,
}

impl SymbolTable {
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
            // TODO: use symbol type for deduplication
            let address: u64 = parts[0].parse()?;
            // TODO: rustc_demangle::demangle
            let symbol: String = parts[2..].join(" ");
            self.sym.push((address, symbol));
        }

        self.sym.sort();

        Ok(())
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
