// Copyright (c) 2017 FaultyRAM
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Platform-specific utilities.

#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
mod posix;
#[cfg(windows)]
mod windows;

#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
pub use self::posix::*;
#[cfg(windows)]
pub use self::windows::*;
