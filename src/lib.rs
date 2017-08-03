// Copyright (c) 2017 FaultyRAM
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Utilities for updating filesystem timestamps, a la touch on Unix.
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", forbid(clippy))]
#![cfg_attr(feature = "clippy", forbid(clippy_internal))]
#![cfg_attr(feature = "clippy", deny(clippy_pedantic))]
#![forbid(warnings)]
#![forbid(anonymous_parameters)]
#![forbid(box_pointers)]
#![forbid(fat_ptr_transmutes)]
#![forbid(missing_docs)]
#![forbid(trivial_casts)]
#![forbid(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![forbid(unused_extern_crates)]
#![forbid(unused_import_braces)]
#![deny(unused_qualifications)]
#![forbid(unused_results)]
#![forbid(variant_size_differences)]

#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
extern crate libc;
#[cfg(windows)]
extern crate kernel32;
#[cfg(windows)]
extern crate winapi;

mod sys;

use std::io;
use std::path::Path;
use std::time::SystemTime;

#[derive(Clone, Copy, Debug)]
/// A builder for updating filesystem timestamps.
pub struct Builder {
    /// The new access timestamp.
    ///
    /// If this is `None`, the access timestamp will not be modified.
    accessed: Option<SystemTime>,
    /// The new modification timestamp.
    ///
    /// If this is `None`, the modification timestamp will not be modified.
    modified: Option<SystemTime>,
}

#[inline]
/// Updates the timestamps for a filesystem path to the current system time.
///
/// If the path does not exist, this will create an empty file at that path.
///
/// If the path refers to a symbolic link, this will follow the symbolic link.
///
/// This wraps the functionality provided by the `Builder` type and is provided for convenience.
/// For more fine-grained control, use `Builder` directly.
pub fn touch<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let now = SystemTime::now();
    Builder::new().accessed(now).modified(now).touch(path)
}

#[inline]
/// Updates the timestamps for a filesystem path to the current system time.
///
/// If the path does not exist, this will create an empty file at that path.
///
/// If the path refers to a symbolic link, this will update the symbolic link.
///
/// This wraps the functionality provided by the `Builder` type and is provided for convenience.
/// For more fine-grained control, use `Builder` directly.
pub fn touch_symlink<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let now = SystemTime::now();
    Builder::new()
        .accessed(now)
        .modified(now)
        .touch_symlink(path)
}

impl Builder {
    #[inline]
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        Self {
            accessed: None,
            modified: None,
        }
    }

    #[inline]
    /// The access timestamp to use when updating timestamps.
    ///
    /// If this method is not called, the access timestamp will not be updated.
    pub fn accessed(&mut self, time: SystemTime) -> &mut Self {
        self.accessed = Some(time);
        self
    }

    #[inline]
    /// The modification timestamp to use when updating timestamps.
    ///
    /// If this method is not called, the modification timestamp will not be updated.
    pub fn modified(&mut self, time: SystemTime) -> &mut Self {
        self.modified = Some(time);
        self
    }

    #[inline]
    /// Updates the timestamps for a filesystem path.
    ///
    /// If the path does not exist, this will create an empty file at that path.
    ///
    /// If the path refers to a symbolic link, this will follow the symbolic link.
    pub fn touch<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.touch_sys(path)
    }

    #[inline]
    /// Updates the timestamps for a filesystem path.
    ///
    /// If the path does not exist, this will create an empty file at that path.
    ///
    /// If the path refers to a symbolic link, this will update the symbolic link.
    pub fn touch_symlink<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.touch_symlink_sys(path)
    }
}

impl Default for Builder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
