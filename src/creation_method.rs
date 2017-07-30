// Copyright (c) 2017 FaultyRAM
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Rules for updating timestamps when a given path does not exist.

use item::Item;
use std::marker::PhantomData;

/// Do not attempt to create an item that does not exist.
pub struct NoCreate;

/// Create an item non-recursively if it does not exist.
pub struct NonRecursive<I: Item>(PhantomData<I>);

/// Create an item recursively if it does not exist.
pub struct Recursive<I: Item>(PhantomData<I>);

/// A trait shared by filesystem creation methods.
pub trait CreationMethod {}

impl CreationMethod for NoCreate {}

impl<I: Item> CreationMethod for NonRecursive<I> {}

impl<I: Item> CreationMethod for Recursive<I> {}
