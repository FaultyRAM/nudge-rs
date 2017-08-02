// Copyright (c) 2017 FaultyRAM
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Unix-specific utilities.

#![allow(unsafe_code)]

use {Builder, Touch, TouchOptions};
use creation_method::{NoCreate, NonRecursive, Recursive};
use item::{Directory, File, Item};
use libc::{self, c_char, c_int, c_long, time_t, timespec, AT_FDCWD, AT_SYMLINK_NOFOLLOW, O_CREAT,
           O_DIRECTORY, O_TRUNC, O_WRONLY, S_IRGRP, S_IROTH, S_IRUSR, S_IWGRP, S_IWOTH, S_IWUSR,
           UTIME_OMIT};
use resolution_method::{FollowSymlinks, ResolutionMethod, UpdateSymlinks};
use std::{io, iter};
use std::fs::DirBuilder;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::DirBuilderExt;
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
/// Updates the timestamps for an existing path, after resolving symbolic links.
fn update_existing_follow<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
    utimensat(builder, path, 0)
}

#[inline]
/// Updates the timestamps for an existing path, without resolving symbolic links.
fn update_existing_update<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
    utimensat(builder, path, AT_SYMLINK_NOFOLLOW)
}

#[inline]
/// Updates the timestamps for a directory that does not yet exist.
fn update_new_dir<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
    FileHandle::open(path, O_DIRECTORY).and_then(|fd| futimens(builder, &fd))
}

#[inline]
/// Updates the timestamps for a file that does not yet exist.
fn update_new_file<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
    FileHandle::open(path, 0).and_then(|fd| futimens(builder, &fd))
}

#[inline]
/// Safely wraps the POSIX `futimens` function.
fn futimens(builder: &Builder, fd: &FileHandle) -> io::Result<()> {
    let times = FileTimes::from_builder(builder);
    if unsafe { libc::futimens(fd.0, times.as_ptr()) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[inline]
/// Safely wraps the POSIX `utimensat` function.
fn utimensat<P: AsRef<Path>>(builder: &Builder, path: P, flag: c_int) -> io::Result<()> {
    let p = into_c_string(path);
    let times = FileTimes::from_builder(builder);
    if unsafe { libc::utimensat(AT_FDCWD, p.as_ptr(), times.as_ptr(), flag) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

impl FileHandle {
    #[inline]
    #[cfg_attr(feature = "clippy", allow(cast_possible_wrap))]
    /// Opens a path.
    pub fn open<P: AsRef<Path>>(path: P, oflag: c_int) -> io::Result<Self> {
        let p = into_c_string(path);
        let fd = unsafe {
            libc::open(
                p.as_ptr(),
                O_WRONLY | O_CREAT | O_TRUNC | oflag,
                (S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH) as c_int,
            )
        };
        if fd <= 0 {
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

impl Touch for TouchOptions<NoCreate, FollowSymlinks> {
    #[inline]
    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
        update_existing_follow(builder, path)
    }
}

impl Touch for TouchOptions<NoCreate, UpdateSymlinks> {
    #[inline]
    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
        update_existing_update(builder, path)
    }
}

impl<R: ResolutionMethod> Touch for TouchOptions<NonRecursive<Directory>, R>
where
    TouchOptions<NoCreate, R>: Touch,
{
    #[inline]
    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
        TouchOptions::<NoCreate, R>::touch(builder, &path)
            .or_else(|_| update_new_dir(builder, path))
    }
}

impl<R: ResolutionMethod> Touch for TouchOptions<NonRecursive<File>, R>
where
    TouchOptions<NoCreate, R>: Touch,
{
    #[inline]
    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
        TouchOptions::<NoCreate, R>::touch(builder, &path)
            .or_else(|_| update_new_file(builder, path))
    }
}

impl<I: Item, R: ResolutionMethod> Touch for TouchOptions<Recursive<I>, R>
where
    TouchOptions<NonRecursive<I>, R>: Touch,
{
    #[inline]
    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
        let rec_res = if let Some(parent) = path.as_ref().parent() {
            DirBuilder::new().recursive(true).mode(0o666).create(parent)
        } else {
            Ok(())
        };
        rec_res.and_then(|_| {
            TouchOptions::<NonRecursive<I>, R>::touch::<P>(builder, path)
        })
    }
}
