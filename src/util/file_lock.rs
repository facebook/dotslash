/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! Wrapper around `fs2::lock_exclusive`.

use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileLockError {
    #[error("failed to create lock file `{0}`")]
    Create(PathBuf, #[source] io::Error),

    #[error("failed to get exclusive lock `{0}`")]
    LockExclusive(PathBuf, #[source] io::Error),
}

#[derive(Debug, Default)]
pub struct FileLock {
    /// If file is Some, then it is holding the lock.
    file: Option<File>,
}

impl FileLock {
    pub fn acquire<P>(path: P) -> Result<FileLock, FileLockError>
    where
        P: AsRef<Path>,
    {
        fn inner(path: &Path) -> Result<FileLock, FileLockError> {
            let lock_file = File::options()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(path)
                .map_err(|e| FileLockError::Create(path.to_path_buf(), e))?;

            fs2::FileExt::lock_exclusive(&lock_file)
                .map_err(|e| FileLockError::LockExclusive(path.to_path_buf(), e))?;

            Ok(FileLock {
                file: Some(lock_file),
            })
        }
        inner(path.as_ref())
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            drop(fs2::FileExt::unlock(&file));
        }
    }
}
