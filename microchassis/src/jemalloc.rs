#![allow(unsafe_code, clippy::expect_used)]

extern crate alloc;

use crate::allocator::OomPanicAllocator;
use alloc::{
    ffi::{CString, NulError},
    fmt,
};
use lazy_static::lazy_static;
use std::{io, os::fd::AsRawFd, ptr};
use tempfile::tempfile;
use tikv_jemalloc_ctl::{raw as mallctl, Error as MallctlError};
use tikv_jemallocator::Jemalloc;

// Simply force jemalloc here as global allocator.
// TODO: document this properly.
#[global_allocator]
static ALLOC: OomPanicAllocator<Jemalloc> = OomPanicAllocator(Jemalloc);

lazy_static! {
    static ref PROF_ACTIVE_MIB: [usize; 2] = {
        let mut mib = [0; 2];
        mallctl::name_to_mib(b"prof.active\0", &mut mib).expect("mib");
        mib
    };
    static ref PROF_DUMP_MIB: [usize; 2] = {
        let mut mib = [0; 2];
        mallctl::name_to_mib(b"prof.dump\0", &mut mib).expect("mib");
        mib
    };
}

/// Sets jemalloc prof.active value.
pub fn set_prof_active(value: bool) -> Result<(), Error> {
    // SAFETY: use correct type (bool) for this mallctl command.
    unsafe { mallctl::write_mib(&*PROF_ACTIVE_MIB, value).map_err(Into::into) }
}

/// Returns jemalloc prof.active value.
pub fn get_prof_active() -> Result<bool, Error> {
    // SAFETY: use correct type (bool) for this mallctl command.
    unsafe { mallctl::read_mib(&*PROF_ACTIVE_MIB).map_err(Into::into) }
}

/// Writes profile dump into file. If a path is not given uses a file name pattern.
/// defined by jemalloc options.
pub fn prof_dump_file(path: Option<&str>) -> Result<(), Error> {
    let ptr = match path {
        Some(s) => CString::new(s)?.into_bytes_with_nul().as_ptr(),
        None => ptr::null(),
    };
    // SAFETY: use correct type (*char+\0) for this mallctl command.
    unsafe { mallctl::write_mib(&*PROF_DUMP_MIB, ptr).map_err(Into::into) }
}

/// Returns a profile dump. Uses [`prof_dump_file`] to write to a temporary
/// file first.
pub fn prof_dump() -> Result<Vec<u8>, Error> {
    let mut file = tempfile()?;
    let path = format!("/proc/self/fd/{fd}", fd = file.as_raw_fd());
    prof_dump_file(Some(path.as_str()))?;
    let mut buf = Vec::new();
    io::copy(&mut file, &mut buf)?;
    Ok(buf)
}

#[derive(thiserror::Error, fmt::Debug)]
pub enum Error {
    #[error("mallctl error: {0}")]
    Mallctl(#[from] MallctlError),

    #[error("NUL byte found error: {0}")]
    Nul(#[from] NulError),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}
