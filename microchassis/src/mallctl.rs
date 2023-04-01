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

#![allow(unsafe_code, clippy::expect_used)]

use lazy_static::lazy_static;
use std::{ffi, fmt, fs, io, mem, ptr};
use tikv_jemalloc_ctl::{raw, Error as MallctlError};
use tikv_jemalloc_sys::mallctlbymib;

// TODO: make OomPanicAllocator optional
#[cfg(feature = "set-jemalloc-global")]
#[global_allocator]
static ALLOC: crate::allocator::OomPanicAllocator<tikv_jemallocator::Jemalloc> =
    crate::allocator::OomPanicAllocator(tikv_jemallocator::Jemalloc);

lazy_static! {
    static ref OPT_PROF_MIB: [usize; 2] = {
        let mut mib = [0; 2];
        raw::name_to_mib(b"opt.prof\0", &mut mib).expect("mib");
        mib
    };
    static ref OPT_LG_PROF_SAMPLE_MIB: [usize; 2] = {
        let mut mib = [0; 2];
        raw::name_to_mib(b"opt.lg_prof_sample\0", &mut mib).expect("mib");
        mib
    };
    static ref PROF_ACTIVE_MIB: [usize; 2] = {
        let mut mib = [0; 2];
        raw::name_to_mib(b"prof.active\0", &mut mib).expect("mib");
        mib
    };
    static ref PROF_DUMP_MIB: [usize; 2] = {
        let mut mib = [0; 2];
        raw::name_to_mib(b"prof.dump\0", &mut mib).expect("mib");
        mib
    };
    static ref PROF_RESET_MIB: [usize; 2] = {
        let mut mib = [0; 2];
        raw::name_to_mib(b"prof.reset\0", &mut mib).expect("mib");
        mib
    };
    static ref PROF_LG_SAMPLE_MIB: [usize; 2] = {
        let mut mib = [0; 2];
        raw::name_to_mib(b"prof.lg_sample\0", &mut mib).expect("mib");
        mib
    };
}

/// Returns jemalloc opt.prof value.
pub fn get_prof_enabled() -> Result<bool, Error> {
    // SAFETY: use correct type (bool) for this mallctl command.
    unsafe { raw::read_mib(&*OPT_PROF_MIB).map_err(Into::into) }
}

/// Returns jemalloc opt.lg_prof_sample value.
pub fn get_lg_prof_sample_opt() -> Result<usize, Error> {
    // SAFETY: use correct type (bool) for this mallctl command.
    unsafe { raw::read_mib(&*OPT_LG_PROF_SAMPLE_MIB).map_err(Into::into) }
}

/// Sets jemalloc prof.active value.
pub fn set_prof_active(value: bool) -> Result<(), Error> {
    // SAFETY: use correct type (bool) for this mallctl command.
    unsafe { raw::write_mib(&*PROF_ACTIVE_MIB, value).map_err(Into::into) }
}

/// Returns jemalloc prof.active value.
pub fn get_prof_active() -> Result<bool, Error> {
    // SAFETY: use correct type (bool) for this mallctl command.
    unsafe { raw::read_mib(&*PROF_ACTIVE_MIB).map_err(Into::into) }
}

/// Sets jemalloc prof.reset value.
#[allow(clippy::option_if_let_else, clippy::borrow_as_ptr)]
pub fn prof_reset(sample: Option<usize>) -> Result<(), Error> {
    let value = match sample {
        Some(mut sample) => &mut sample as *mut _,
        None => ptr::null_mut(),
    };
    // SAFETY: only use correct type (*size_t) for this mallctl command.
    unsafe { write_mib_ptr(&*PROF_RESET_MIB, value).map_err(Into::into) }
}

/// Returns jemalloc prof.lg_sample value.
pub fn get_prof_lg_sample() -> Result<usize, Error> {
    // SAFETY: use correct type (size_t) for this mallctl command.
    unsafe { raw::read_mib(&*PROF_LG_SAMPLE_MIB).map_err(Into::into) }
}

unsafe fn write_mib_ptr<T>(mib: &[usize], value: *mut T) -> Result<(), Error> {
    match mallctlbymib(
        mib.as_ptr(),
        mib.len(),
        ptr::null_mut(),
        ptr::null_mut(),
        value.cast(),
        mem::size_of::<T>(),
    ) {
        0 => Ok(()),
        c => Err(Error::MallctlCode(c)),
    }
}

/// Writes profile dump into file. If a path is not given uses a file name pattern.
/// defined by jemalloc options.
pub fn prof_dump(path: Option<&str>) -> Result<Option<Vec<u8>>, Error> {
    let ptr = match path {
        Some(s) => ffi::CString::new(s)?.into_bytes_with_nul().as_ptr(),
        None => ptr::null(),
    };

    // SAFETY: must use correct type (*char+\0) for this mallctl command.
    unsafe {
        raw::write_mib(&*PROF_DUMP_MIB, ptr)?;
    }

    match path {
        Some(path) => {
            let mut f = fs::File::open(path)?;
            let mut buf = Vec::new();
            io::copy(&mut f, &mut buf)?;
            Ok(Some(buf))
        }
        None => Ok(None),
    }
}

#[derive(thiserror::Error, fmt::Debug)]
pub enum Error {
    #[error("mallctl error: {0}")]
    Mallctl(#[from] MallctlError),

    #[error("mallctl error code: {0}")]
    MallctlCode(ffi::c_int),

    #[error("NUL byte found error: {0}")]
    Nul(#[from] ffi::NulError),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_prof_active() {
        // _RJEM_MALLOC_CONF=prof:true,prof_active:false
        assert!(get_prof_enabled().expect("get_prof_enabled"));

        assert!(!get_prof_active().expect("get_prof_active"));
        set_prof_active(true).expect("set_prof_active");
        assert!(get_prof_active().expect("get_prof_active"));
        set_prof_active(false).expect("set_prof_active");
        assert!(!get_prof_active().expect("get_prof_active"));
    }

    #[test]
    #[ignore]
    fn test_prof_reset() {
        // _RJEM_MALLOC_CONF=prof:true,prof_active:false,lg_prof_sample:10
        assert!(get_prof_enabled().expect("get_prof_enabled"));
        assert_eq!(10, get_lg_prof_sample_opt().expect("get_lg_sample_opt"));
        assert_eq!(10, get_prof_lg_sample().expect("get_prof_lg_sample"));

        prof_reset(None).expect("prof_reset");
        assert_eq!(10, get_prof_lg_sample().expect("get_prof_lg_sample"));
        prof_reset(Some(8)).expect("prof_reset");
        assert_eq!(8, get_prof_lg_sample().expect("get_prof_lg_sample"));
        prof_reset(None).expect("prof_reset");
        assert_eq!(8, get_prof_lg_sample().expect("get_prof_lg_sample"));

        assert_eq!(10, get_lg_prof_sample_opt().expect("get_lg_sample_opt"));
    }
}
