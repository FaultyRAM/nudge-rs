// Copyright (c) 2017 FaultyRAM
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Filesystem items, such as directories and files.

/// A directory.
pub struct Directory;

/// A file.
pub struct File;

/// A trait shared by filesystem items.
pub trait Item {}

impl Item for Directory {}

impl Item for File {}
