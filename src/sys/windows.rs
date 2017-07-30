// Copyright (c) 2017 FaultyRAM
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Windows-specific utilities.

#![cfg_attr(feature = "clippy", allow(cast_possible_truncation))]
#![allow(unsafe_code)]

use {Builder, Touch, TouchOptions};
use creation_method::{NoCreate, NonRecursive, Recursive};
use item::{Directory, File, Item};
use kernel32;
use resolution_method::{FollowSymlinks, ResolutionMethod, UpdateSymlinks};
use std::fs::DirBuilder;
use std::io;
use std::path::Path;
use std::ptr;
use std::os::windows::ffi::OsStrExt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use winapi::{DWORD, FILETIME, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
             FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_WRITE_ATTRIBUTES, HANDLE,
             INVALID_HANDLE_VALUE, OPEN_ALWAYS, OPEN_EXISTING, WCHAR};

/// A safe wrapper around a Windows file handle.
struct FileHandle(HANDLE);

#[derive(Clone, Copy)]
/// Holds Windows timestamps for a file.
struct FileTimes {
    /// The access timestamp.
    accessed: FILETIME,
    /// The modification timestamp.
    modified: FILETIME,
}

#[inline]
/// Converts a path into a Windows wide string for use in FFI calls.
fn into_wide_string<P: AsRef<Path>>(path: P) -> Vec<WCHAR> {
    path.as_ref().as_os_str().encode_wide().collect()
}

impl FileHandle {
    #[inline]
    /// Creates a file handle to a path with the given flags.
    pub fn open<P: AsRef<Path>>(path: P, disp: DWORD, flags: DWORD) -> io::Result<FileHandle> {
        let p = into_wide_string(path);
        let fd = unsafe {
            kernel32::CreateFileW(
                p.as_ptr(),
                FILE_WRITE_ATTRIBUTES,
                FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
                ptr::null_mut(),
                disp,
                FILE_FLAG_BACKUP_SEMANTICS | flags,
                ptr::null_mut(),
            )
        };
        if fd == INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(FileHandle(fd))
        }
    }

    #[inline]
    /// Updates the timestamps for a file.
    pub fn update_timestamps(&mut self, times: FileTimes) -> io::Result<()> {
        if unsafe {
            kernel32::SetFileTime(self.0, ptr::null(), times.accessed(), times.modified())
        } == 0
        {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Drop for FileHandle {
    #[inline]
    fn drop(&mut self) {
        if unsafe { kernel32::CloseHandle(self.0) } == 0 {
            panic!("{}", io::Error::last_os_error());
        }
    }
}

impl FileTimes {
    #[inline]
    /// Obtains a set of Windows timestamps from a `Builder`.
    pub fn from_builder(builder: &Builder) -> Self {
        FileTimes {
            accessed: Self::systemtime_into_filetime(builder.accessed),
            modified: Self::systemtime_into_filetime(builder.modified),
        }
    }

    #[inline]
    /// Returns a reference to the access timestamp.
    pub fn accessed(&self) -> &FILETIME {
        &self.accessed
    }

    #[inline]
    /// Returns a reference to the modification timestamp.
    pub fn modified(&self) -> &FILETIME {
        &self.modified
    }

    #[inline]
    /// Converts a Rust timestamp into a Windows timestamp.
    fn systemtime_into_filetime(time: Option<SystemTime>) -> FILETIME {
        if let Some(t) = time {
            // Windows does not use the Unix epoch! The Windows epoch is January 1, 1601 (UTC).
            let unix_epoch = Duration::from_secs(11644473600);
            let duration = match t.duration_since(UNIX_EPOCH) {
                Ok(d) => d + unix_epoch,
                Err(e) => unix_epoch - e.duration(),
            };
            // Windows timestamps have a resolution of 100 nanoseconds.
            let nanos = duration.as_secs() * 10000000 + (duration.subsec_nanos() / 100) as u64;
            FILETIME {
                dwLowDateTime: nanos as DWORD,
                dwHighDateTime: (nanos >> 32) as DWORD,
            }
        } else {
            FILETIME {
                dwLowDateTime: 0xFFFFFFFF,
                dwHighDateTime: 0xFFFFFFFF,
            }
        }
    }
}

impl Touch for TouchOptions<NoCreate, FollowSymlinks> {
    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
        let times = FileTimes::from_builder(builder);
        FileHandle::open(path, OPEN_EXISTING, 0).and_then(|mut f| f.update_timestamps(times))
    }
}

impl Touch for TouchOptions<NoCreate, UpdateSymlinks> {
    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
        let times = FileTimes::from_builder(builder);
        FileHandle::open(path, OPEN_EXISTING, FILE_FLAG_OPEN_REPARSE_POINT)
            .and_then(|mut f| f.update_timestamps(times))
    }
}

impl<R: ResolutionMethod> Touch for TouchOptions<NonRecursive<Directory>, R>
where
    TouchOptions<NoCreate, R>: Touch,
{
    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
        let p = into_wide_string(&path);
        let _ = unsafe { kernel32::CreateDirectoryW(p.as_ptr(), ptr::null_mut()) };
        TouchOptions::<NoCreate, R>::touch::<P>(builder, path)
    }
}

impl Touch for TouchOptions<NonRecursive<File>, FollowSymlinks> {
    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
        let times = FileTimes::from_builder(builder);
        FileHandle::open(path, OPEN_ALWAYS, 0).and_then(|mut f| f.update_timestamps(times))
    }
}

impl Touch for TouchOptions<NonRecursive<File>, UpdateSymlinks> {
    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
        let times = FileTimes::from_builder(builder);
        FileHandle::open(path, OPEN_ALWAYS, FILE_FLAG_OPEN_REPARSE_POINT)
            .and_then(|mut f| f.update_timestamps(times))
    }
}

impl<I: Item, R: ResolutionMethod> Touch for TouchOptions<Recursive<I>, R>
where
    TouchOptions<NoCreate, R>: Touch,
{
    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()> {
        let rec_res = if let Some(parent) = path.as_ref().parent() {
            DirBuilder::new().recursive(true).create(parent)
        } else {
            Ok(())
        };
        rec_res.and_then(|_| TouchOptions::<NoCreate, R>::touch::<P>(builder, path))
    }
}
