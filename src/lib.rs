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

#[derive(Clone, Debug)]
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
    /// Whether to follow symbolic links.
    follow_symlinks: bool,
    /// What to create if a path does not exist.
    creation_target: CreationTarget,
}

#[derive(Clone, Debug)]
/// What to create if a path does not exist.
pub enum CreationTarget {
    /// Do not create anything.
    None,
    /// Create a file.
    File,
}

impl Builder {
    #[inline]
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        Self {
            accessed: None,
            modified: None,
            follow_symlinks: false,
            creation_target: CreationTarget::None,
        }
    }

    #[inline]
    /// Specifies the access timestamp to use when updating timestamps.
    ///
    /// If this is `None` (the default), the access timestamp will not be updated.
    pub fn accessed(&mut self, time: Option<SystemTime>) -> &mut Self {
        self.accessed = time;
        self
    }

    #[inline]
    /// Specifies the modification timestamp to use when updating timestamps.
    ///
    /// If this is `None` (the default), the modification timestamp will not be updated.
    pub fn modified(&mut self, time: Option<SystemTime>) -> &mut Self {
        self.modified = time;
        self
    }

    #[inline]
    /// Specifies whether to follow symbolic links.
    ///
    /// If this is `false` (the default) and a path refers to a symbolic link, the symbolic link
    /// will be updated instead of the path it refers to.
    pub fn follow_symlinks(&mut self, follow: bool) -> &mut Self {
        self.follow_symlinks = follow;
        self
    }

    #[inline]
    /// Specifies what to create if a path does not exist.
    ///
    /// This is non-recursive, i.e. if any parent directories do not exist, creation will fail.
    ///
    /// By default, nothing will be created.
    pub fn creation_target(&mut self, target: CreationTarget) -> &mut Self {
        self.creation_target = target;
        self
    }

    #[inline]
    /// Updates the timestamps for a filesystem path, using the options given to a builder.
    pub fn touch<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.touch_sys(path)
    }
}

impl Default for Builder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CreationTarget {
    #[inline]
    fn default() -> Self {
        CreationTarget::None
    }
}
