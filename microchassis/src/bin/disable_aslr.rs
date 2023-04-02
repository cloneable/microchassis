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

use std::{env, ffi::CString, io, iter, ptr};

fn main() -> io::Result<()> {
    if env::args().len() < 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "no command to run specified"));
    }

    let args: Vec<_> = env::args()
        .skip(1)
        .map(|arg| CString::new(arg.as_str()).expect("no zero in arg"))
        .collect();

    let envvars: Vec<_> = env::vars()
        .map(|(name, value)| CString::new(format!("{name}={value}")).expect("no zero in env var"))
        .collect();

    exec(args, envvars)
}

#[cfg(target_os = "macos")]
fn exec(args: Vec<CString>, envvars: Vec<CString>) -> io::Result<()> {
    let args: Vec<_> =
        args.iter().map(|arg| arg.as_ptr() as *mut _).chain(iter::once(ptr::null_mut())).collect();
    let envvars: Vec<_> = envvars
        .iter()
        .map(|arg| arg.as_ptr() as *mut _)
        .chain(iter::once(ptr::null_mut()))
        .collect();

    let mut attrs = std::mem::MaybeUninit::<libc::posix_spawnattr_t>::uninit();
    err(unsafe { libc::posix_spawnattr_init(attrs.as_mut_ptr()) })?;
    err(unsafe {
        libc::posix_spawnattr_setflags(attrs.as_mut_ptr(), libc::POSIX_SPAWN_SETEXEC as i16 | 0x100)
    })?;

    err(unsafe {
        libc::posix_spawnp(
            ptr::null_mut(),
            args[0],
            ptr::null_mut(),
            attrs.as_ptr(),
            args.as_ptr(),
            envvars.as_ptr(),
        )
    })?;

    unreachable!()
}

// TODO: drop in favor of mentioning `setarch -R` in the docs?
//       and /proc/sys/kernel/randomize_va_space
#[cfg(target_os = "linux")]
fn exec(args: Vec<CString>, envvars: Vec<CString>) -> io::Result<()> {
    let args: Vec<_> = args.iter().map(|arg| arg.as_ptr()).chain(iter::once(ptr::null())).collect();
    let envvars: Vec<_> =
        envvars.iter().map(|arg| arg.as_ptr()).chain(iter::once(ptr::null())).collect();

    err(unsafe { libc::personality(libc::ADDR_NO_RANDOMIZE as u64) })?;
    err(unsafe { libc::execvpe(args[0], args.as_ptr(), envvars.as_ptr()) })?;

    unreachable!()
}

fn err(ret: i32) -> io::Result<i32> {
    if ret >= 0 {
        Ok(ret)
    } else {
        Err(io::Error::last_os_error())
    }
}
