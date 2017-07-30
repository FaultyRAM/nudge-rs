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
#![cfg_attr(feature = "clippy", forbid(clippy_pedantic))]
#![cfg_attr(feature = "clippy", forbid(clippy_restrictions))]
#![forbid(warnings)]
#![forbid(anonymous_parameters)]
#![forbid(box_pointers)]
#![forbid(fat_ptr_transmutes)]
#![forbid(missing_docs)]
#![forbid(trivial_casts)]
#![forbid(trivial_numeric_casts)]
#![forbid(unsafe_code)]
#![forbid(unused_extern_crates)]
#![forbid(unused_import_braces)]
#![deny(unused_qualifications)]
#![forbid(unused_results)]
#![forbid(variant_size_differences)]

pub mod creation_method;
pub mod item;
mod sys;

/// Re-exports of commonly-used functionality.
pub mod prelude {
    pub use Builder;
    pub use creation_method::{CreationMethod, NoCreate, NonRecursive, Recursive};
    pub use item::{Directory, File, Item};
}

use self::creation_method::CreationMethod;
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

/// Updates the timestamps for a filesystem path to the current system time, after resolving
/// symbolic links.
///
/// If the path does not exist, the specified creation method will be used to create the path.
///
/// This wraps the functionality provided by the `Builder` type and is provided for convenience.
/// For more fine-grained control, use `Builder` directly.
pub fn touch<M: CreationMethod, P: AsRef<Path>>(path: P) -> io::Result<()> {
    let now = SystemTime::now();
    Builder::new()
        .accessed(now)
        .modified(now)
        .touch::<M, P>(path)
}

/// Updates the timestamps for a filesystem path to the current system time, without resolving
/// symbolic links.
///
/// If the path does not exist, the specified creation method will be used to create the path.
///
/// This wraps the functionality provided by the `Builder` type and is provided for convenience.
/// For more fine-grained control, use `Builder` directly.
pub fn touch_symlink<M: CreationMethod, P: AsRef<Path>>(path: P) -> io::Result<()> {
    let now = SystemTime::now();
    Builder::new()
        .accessed(now)
        .modified(now)
        .touch_symlink::<M, P>(path)
}

impl Builder {
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        Self {
            accessed: None,
            modified: None,
        }
    }

    /// The access timestamp to use when updating timestamps.
    ///
    /// If this method is not called, the access timestamp will not be updated.
    pub fn accessed(&mut self, time: SystemTime) -> &mut Self {
        self.accessed = Some(time);
        self
    }

    /// The modification timestamp to use when updating timestamps.
    ///
    /// If this method is not called, the modification timestamp will not be updated.
    pub fn modified(&mut self, time: SystemTime) -> &mut Self {
        self.modified = Some(time);
        self
    }

    /// Updates the timestamps for a filesystem path, after resolving symbolic links.
    ///
    /// If the path does not exist, the specified creation method will be used to create the path.
    pub fn touch<M: CreationMethod, P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        sys::touch_existing(self, &path).or_else(|e| if e.kind() == io::ErrorKind::NotFound {
            M::touch_new::<P>(self, path)
        } else {
            Err(e)
        })
    }

    /// Updates the timestamps for a filesystem path, without resolving symbolic links.
    ///
    /// If the path does not exist, the specified creation method will be used to create the path.
    pub fn touch_symlink<M: CreationMethod, P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        sys::touch_existing_symlink(self, &path).or_else(
            |e| if e.kind() == io::ErrorKind::NotFound {
                M::touch_new::<P>(self, path)
            } else {
                Err(e)
            },
        )
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
