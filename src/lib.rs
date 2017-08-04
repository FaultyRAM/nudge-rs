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
#[cfg(test)]
extern crate tempdir;

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
            creation_target: CreationTarget::default(),
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

#[cfg(test)]
mod tests {
    use {Builder, CreationTarget};
    use std::fs::{self, OpenOptions};
    use std::io;
    #[cfg(unix)]
    use std::os::unix;
    #[cfg(windows)]
    use std::os::windows;
    use std::path::{Path, PathBuf};
    use std::time::SystemTime;
    use tempdir::TempDir;

    struct TestHelper(TempDir);

    fn file_path<P: AsRef<Path>>(prefix: P) -> PathBuf {
        prefix.as_ref().join("file.txt")
    }

    fn directory_path<P: AsRef<Path>>(prefix: P) -> PathBuf {
        prefix.as_ref().join("directory")
    }

    fn symlink_file_path<P: AsRef<Path>>(prefix: P) -> PathBuf {
        prefix.as_ref().join("symlink-file.txt")
    }

    fn symlink_directory_path<P: AsRef<Path>>(prefix: P) -> PathBuf {
        prefix.as_ref().join("symlink-directory")
    }

    fn times<P: AsRef<Path>>(path: P) -> (SystemTime, SystemTime) {
        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => panic!(
                "could not obtain metadata for {}: {}",
                path.as_ref().display(),
                e
            ),
        };
        (
            metadata
                .accessed()
                .expect("atime not supported on this platform"),
            metadata
                .modified()
                .expect("mtime not supported on this platform"),
        )
    }

    fn symlink_times<P: AsRef<Path>>(path: P) -> (SystemTime, SystemTime) {
        let metadata = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => panic!(
                "could not obtain metadata for {}: {}",
                path.as_ref().display(),
                e
            ),
        };
        (
            metadata
                .accessed()
                .expect("atime not supported on this platform"),
            metadata
                .modified()
                .expect("mtime not supported on this platform"),
        )
    }

    fn touch<P: AsRef<Path>>(builder: &Builder, path: P) {
        if let Err(e) = builder.touch(path) {
            panic!("`Builder::touch` failed: {}", e);
        }
    }

    impl TestHelper {
        pub fn new() -> TestHelper {
            match TempDir::new("nudge-rs_test") {
                Ok(td) => TestHelper(td),
                Err(e) => panic!("could not create temporary directory: {}", e),
            }
        }

        pub fn create_top_level_file(&self) -> PathBuf {
            let path = file_path(self.0.path());
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => path,
                Err(e) => panic!("could not create top-level file: {}", e),
            }
        }

        pub fn create_top_level_directory(&self) -> PathBuf {
            let path = directory_path(self.0.path());
            match fs::create_dir(&path) {
                Ok(_) => path,
                Err(e) => panic!("could not create top-level directory: {}", e),
            }
        }

        #[cfg(unix)]
        pub fn create_top_level_symlink_file(&self) -> PathBuf {
            let src = file_path(self.0.path());
            let dst = symlink_file_path(self.0.path());
            match unix::fs::symlink(src, &dst) {
                Ok(_) => dst,
                Err(e) => panic!("could not create file symbolic link: {}", e),
            }
        }

        #[cfg(unix)]
        pub fn create_top_level_symlink_directory(&self) -> PathBuf {
            let src = directory_path(self.0.path());
            let dst = symlink_directory_path(self.0.path());
            match unix::fs::symlink(src, &dst) {
                Ok(_) => dst,
                Err(e) => panic!("could not create directory symbolic link: {}", e),
            }
        }

        #[cfg(windows)]
        pub fn create_top_level_symlink_file(&self) -> PathBuf {
            let src = file_path(self.0.path());
            let dst = symlink_file_path(self.0.path());
            match windows::fs::symlink_file(src, &dst) {
                Ok(_) => dst,
                Err(e) => panic!("could not create file symbolic link: {}", e),
            }
        }

        #[cfg(windows)]
        pub fn create_top_level_symlink_directory(&self) -> PathBuf {
            let src = directory_path(self.0.path());
            let dst = symlink_directory_path(self.0.path());
            match windows::fs::symlink_dir(src, &dst) {
                Ok(_) => dst,
                Err(e) => panic!("could not create directory symbolic link: {}", e),
            }
        }

        pub fn nonexisting_file_path(&self) -> PathBuf {
            self.0.path().join("nonexisting-file-path.txt")
        }
    }

    #[test]
    fn existing_file_noupdate() {
        let helper = TestHelper::new();
        let file_path = helper.create_top_level_file();
        let old_times = times(&file_path);
        let builder = Builder::new();
        touch(&builder, &file_path);
        assert_eq!(old_times, times(file_path));
    }

    #[test]
    fn existing_file_atime() {
        let helper = TestHelper::new();
        let file_path = helper.create_top_level_file();
        let (_, old_mtime) = times(&file_path);
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder.accessed(Some(now));
        touch(&builder, &file_path);
        assert_eq!((now, old_mtime), times(file_path));
    }

    #[test]
    fn existing_file_mtime() {
        let helper = TestHelper::new();
        let file_path = helper.create_top_level_file();
        let (old_atime, _) = times(&file_path);
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder.modified(Some(now));
        touch(&builder, &file_path);
        assert_eq!((old_atime, now), times(file_path));
    }

    #[test]
    fn existing_file_times() {
        let helper = TestHelper::new();
        let file_path = helper.create_top_level_file();
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder.accessed(Some(now)).modified(Some(now));
        touch(&builder, &file_path);
        assert_eq!((now, now), times(file_path));
    }

    #[test]
    fn existing_directory_noupdate() {
        let helper = TestHelper::new();
        let dir_path = helper.create_top_level_directory();
        let old_times = times(&dir_path);
        let builder = Builder::new();
        touch(&builder, &dir_path);
        assert_eq!(old_times, times(dir_path));
    }

    #[test]
    fn existing_directory_atime() {
        let helper = TestHelper::new();
        let dir_path = helper.create_top_level_directory();
        let (_, old_mtime) = times(&dir_path);
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder.accessed(Some(now));
        touch(&builder, &dir_path);
        assert_eq!((now, old_mtime), times(dir_path));
    }

    #[test]
    fn existing_directory_mtime() {
        let helper = TestHelper::new();
        let dir_path = helper.create_top_level_directory();
        let (old_atime, _) = times(&dir_path);
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder.modified(Some(now));
        touch(&builder, &dir_path);
        assert_eq!((old_atime, now), times(dir_path));
    }

    #[test]
    fn existing_directory_times() {
        let helper = TestHelper::new();
        let dir_path = helper.create_top_level_directory();
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder.accessed(Some(now)).modified(Some(now));
        touch(&builder, &dir_path);
        assert_eq!((now, now), times(dir_path));
    }

    #[test]
    fn follow_symlink() {
        let helper = TestHelper::new();
        let file_path = helper.create_top_level_file();
        let sym_path = helper.create_top_level_symlink_file();
        let sym_old_times = symlink_times(&sym_path);
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder
            .accessed(Some(now))
            .modified(Some(now))
            .follow_symlinks(true);
        touch(&builder, &sym_path);
        assert_eq!((now, now), times(file_path));
        assert_eq!(sym_old_times, symlink_times(sym_path));
    }

    #[test]
    fn symlink_file_noupdate() {
        let helper = TestHelper::new();
        let sym_path = helper.create_top_level_symlink_file();
        let old_times = symlink_times(&sym_path);
        let builder = Builder::new();
        touch(&builder, &sym_path);
        assert_eq!(old_times, symlink_times(sym_path));
    }

    #[test]
    fn symlink_file_atime() {
        let helper = TestHelper::new();
        let sym_path = helper.create_top_level_symlink_file();
        let (_, old_mtime) = symlink_times(&sym_path);
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder.accessed(Some(now));
        touch(&builder, &sym_path);
        assert_eq!((now, old_mtime), symlink_times(sym_path));
    }

    #[test]
    fn symlink_file_mtime() {
        let helper = TestHelper::new();
        let sym_path = helper.create_top_level_symlink_file();
        let (old_atime, _) = symlink_times(&sym_path);
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder.modified(Some(now));
        touch(&builder, &sym_path);
        assert_eq!((old_atime, now), symlink_times(sym_path));
    }

    #[test]
    fn symlink_file_times() {
        let helper = TestHelper::new();
        let sym_path = helper.create_top_level_symlink_file();
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder.accessed(Some(now)).modified(Some(now));
        touch(&builder, &sym_path);
        assert_eq!((now, now), symlink_times(sym_path));
    }

    #[test]
    fn symlink_directory_noupdate() {
        let helper = TestHelper::new();
        let sym_path = helper.create_top_level_symlink_directory();
        let old_times = symlink_times(&sym_path);
        let builder = Builder::new();
        touch(&builder, &sym_path);
        assert_eq!(old_times, symlink_times(sym_path));
    }

    #[test]
    fn symlink_directory_atime() {
        let helper = TestHelper::new();
        let sym_path = helper.create_top_level_symlink_directory();
        let (_, old_mtime) = symlink_times(&sym_path);
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder.accessed(Some(now));
        touch(&builder, &sym_path);
        assert_eq!((now, old_mtime), symlink_times(sym_path));
    }

    #[test]
    fn symlink_directory_mtime() {
        let helper = TestHelper::new();
        let sym_path = helper.create_top_level_symlink_directory();
        let (old_atime, _) = symlink_times(&sym_path);
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder.modified(Some(now));
        touch(&builder, &sym_path);
        assert_eq!((old_atime, now), symlink_times(sym_path));
    }

    #[test]
    fn symlink_directory_times() {
        let helper = TestHelper::new();
        let sym_path = helper.create_top_level_symlink_directory();
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder.accessed(Some(now)).modified(Some(now));
        touch(&builder, &sym_path);
        assert_eq!((now, now), symlink_times(sym_path));
    }

    #[test]
    fn nocreate() {
        let helper = TestHelper::new();
        match Builder::new().touch(helper.nonexisting_file_path()) {
            Ok(_) => panic!("`Builder::touch` succeeded"),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => panic!("`Builder::touch` failed with an unexpected error: {}", e),
        }
    }

    #[test]
    fn new_file_noupdate() {
        let helper = TestHelper::new();
        let mut builder = Builder::new();
        let _ = builder.creation_target(CreationTarget::File);
        touch(&builder, helper.nonexisting_file_path());
    }

    #[test]
    fn new_file_atime() {
        let helper = TestHelper::new();
        let file_path = helper.nonexisting_file_path();
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder
            .accessed(Some(now))
            .creation_target(CreationTarget::File);
        touch(&builder, &file_path);
        let (new_atime, _) = times(file_path);
        assert_eq!(now, new_atime);
    }

    #[test]
    fn new_file_mtime() {
        let helper = TestHelper::new();
        let file_path = helper.nonexisting_file_path();
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder
            .modified(Some(now))
            .creation_target(CreationTarget::File);
        touch(&builder, &file_path);
        let (_, new_mtime) = times(file_path);
        assert_eq!(now, new_mtime);
    }

    #[test]
    fn new_file_times() {
        let helper = TestHelper::new();
        let file_path = helper.nonexisting_file_path();
        let now = SystemTime::now();
        let mut builder = Builder::new();
        let _ = builder
            .accessed(Some(now))
            .modified(Some(now))
            .creation_target(CreationTarget::File);
        touch(&builder, &file_path);
        assert_eq!((now, now), times(file_path));
    }
}
