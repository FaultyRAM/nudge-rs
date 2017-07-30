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

#[cfg(windows)]
extern crate kernel32;
#[cfg(windows)]
extern crate winapi;

pub mod creation_method;
pub mod item;
pub mod resolution_method;
mod sys;

/// Re-exports of commonly-used functionality.
pub mod prelude {
    pub use Builder;
    pub use creation_method::{CreationMethod, NoCreate, NonRecursive, Recursive};
    pub use item::{Directory, File, Item};
    pub use resolution_method::{FollowSymlinks, ResolutionMethod, UpdateSymlinks};
}

use self::creation_method::CreationMethod;
use self::resolution_method::ResolutionMethod;
use std::io;
use std::marker::PhantomData;
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

#[doc(hidden)]
/// Options for controlling behaviour while updating timestamps.
pub struct TouchOptions<A: CreationMethod, B: ResolutionMethod>(PhantomData<A>, PhantomData<B>);

#[doc(hidden)]
/// Provides an overloadable `touch` method for `TouchOptions`.
pub trait Touch {
    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()>;
}

/// Updates the timestamps for a filesystem path to the current system time.
///
/// If the path does not exist, the specified creation method will be used to create the path.
///
/// If symbolic links are encountered, the specified resolution method will be used to determine
/// how to resolve the path or, alternatively, whether to update the symbolic link itself.
///
/// This wraps the functionality provided by the `Builder` type and is provided for convenience.
/// For more fine-grained control, use `Builder` directly.
pub fn touch<A: CreationMethod, B: ResolutionMethod, C: AsRef<Path>>(path: C) -> io::Result<()>
where
    TouchOptions<A, B>: Touch,
{
    let now = SystemTime::now();
    Builder::new()
        .accessed(now)
        .modified(now)
        .touch::<A, B, C>(path)
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

    /// Updates the timestamps for a filesystem path.
    ///
    /// If the path does not exist, the specified creation method will be used to create the path.
    pub fn touch<A: CreationMethod, B: ResolutionMethod, C: AsRef<Path>>(
        &self,
        path: C,
    ) -> io::Result<()>
    where
        TouchOptions<A, B>: Touch,
    {
        TouchOptions::<A, B>::touch::<C>(self, path)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
