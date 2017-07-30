// Copyright (c) 2017 FaultyRAM
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Rules for updating timestamps when a given path does not exist.

use Builder;
use item::Item;
use std::io;
use std::marker::PhantomData;
use std::path::Path;

/// Do not attempt to create an item that does not exist.
pub struct NoCreate;

/// Create an item non-recursively if it does not exist.
pub struct NonRecursive<I: Item>(PhantomData<I>);

/// Create an item recursively if it does not exist.
pub struct Recursive<I: Item>(PhantomData<I>);

/// A trait shared by filesystem creation methods.
pub trait CreationMethod {
    #[doc(hidden)]
    /// Updates the timestamps for a filesystem path that does not yet exist.
    fn touch_new<P: AsRef<Path>>(builder: &Builder, path: P) -> io::Result<()>;
}
