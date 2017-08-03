// Copyright (c) 2017 FaultyRAM
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Unix-specific utilities.

#![allow(unsafe_code)]

use {Builder, CreationTarget};
use libc::{self, c_char, c_int, c_long, time_t, timespec, AT_FDCWD, AT_SYMLINK_NOFOLLOW, O_CREAT,
           O_TRUNC, O_WRONLY, S_IRGRP, S_IROTH, S_IRUSR, S_IWGRP, S_IWOTH, S_IWUSR, UTIME_OMIT};
use std::{io, iter};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// A safe wrapper around a file descriptor.
struct FileHandle(c_int);

/// Holds Unix timestamps for a file.
struct FileTimes([timespec; 2]);

#[inline]
#[cfg_attr(feature = "clippy", allow(cast_possible_wrap))]
/// Converts a path into a C string for use in FFI calls.
fn into_c_string<P: AsRef<Path>>(path: P) -> Vec<c_char> {
    path.as_ref()
        .as_os_str()
        .as_bytes()
        .iter()
        .map(|c| *c as c_char)
        .chain(iter::once(0))
        .collect()
}

#[inline]
/// Safely wraps the POSIX `futimens` function.
fn futimens(fd: &FileHandle, times: *const timespec) -> io::Result<()> {
    if unsafe { libc::futimens(fd.0, times) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[inline]
/// Safely wraps the POSIX `utimensat` function.
fn utimensat(path: *const c_char, times: *const timespec, flag: c_int) -> io::Result<()> {
    if unsafe { libc::utimensat(AT_FDCWD, path, times, flag) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

impl FileHandle {
    #[inline]
    #[cfg_attr(feature = "clippy", allow(cast_possible_wrap))]
    /// Opens a path.
    pub fn open(path: *const c_char) -> io::Result<Self> {
        let fd = unsafe {
            libc::open(
                path,
                O_WRONLY | O_CREAT | O_TRUNC,
                (S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH) as c_int,
            )
        };
        if fd >= 0 {
            Ok(FileHandle(fd))
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

impl Drop for FileHandle {
    #[inline]
    fn drop(&mut self) {
        if unsafe { libc::close(self.0) } != 0 {
            panic!(
                "could not close file descriptor: {}",
                io::Error::last_os_error()
            );
        }
    }
}

impl FileTimes {
    #[inline]
    /// Obtains a set of Unix timestamps from a `Builder`.
    pub fn from_builder(builder: &Builder) -> Self {
        FileTimes([
            Self::systemtime_into_filetime(builder.accessed),
            Self::systemtime_into_filetime(builder.modified),
        ])
    }

    #[inline]
    /// Returns a raw pointer suitable for use in time-related functions.
    pub fn as_ptr(&self) -> *const timespec {
        &self.0[0]
    }

    #[inline]
    #[cfg_attr(feature = "clippy", allow(cast_possible_wrap))]
    /// Converts a Rust timestamp into a Unix timestamp.
    fn systemtime_into_filetime(time: Option<SystemTime>) -> timespec {
        if let Some(t) = time {
            match t.duration_since(UNIX_EPOCH) {
                Ok(d) => timespec {
                    tv_sec: d.as_secs() as time_t,
                    tv_nsec: d.subsec_nanos() as c_long,
                },
                Err(e) => timespec {
                    tv_sec: -(e.duration().as_secs() as time_t),
                    tv_nsec: -(e.duration().subsec_nanos() as c_long),
                },
            }
        } else {
            timespec {
                tv_sec: 0,
                tv_nsec: UTIME_OMIT,
            }
        }
    }
}

impl Builder {
    #[inline]
    /// Implementation details.
    pub(crate) fn touch_sys<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let p = into_c_string(path);
        let times = FileTimes::from_builder(self);
        let utimensat_flag = if self.follow_symlinks {
            0
        } else {
            AT_SYMLINK_NOFOLLOW
        };
        utimensat(p.as_ptr(), times.as_ptr(), utimensat_flag)
            .or_else(|e| if e.kind() == io::ErrorKind::NotFound {
                match self.creation_target {
                    CreationTarget::None => Err(e),
                    CreationTarget::File => {
                        FileHandle::open(p.as_ptr()).and_then(|fd| futimens(&fd, times.as_ptr()))
                    }
                }
            } else {
                Err(e)
            })
    }
}
