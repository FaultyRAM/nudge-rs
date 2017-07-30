// Copyright (c) 2017 FaultyRAM
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Platform-specific utilities.

use Builder;
use std::io;
use std::path::Path;

/// Updates the timestamps for an already existing path, after resolving symbolic links.
///
/// # Failures
///
/// Fails if the path does not exist.
pub fn touch_existing<P: AsRef<Path>>(_builder: &Builder, _path: P) -> io::Result<()> {
    unimplemented!()
}

/// Updates the timestamps for an already existing path, without resolving symbolic links.
///
/// # Failures
///
/// Fails if the path does not exist.
pub fn touch_existing_symlink<P: AsRef<Path>>(_builder: &Builder, _path: P) -> io::Result<()> {
    unimplemented!()
}
