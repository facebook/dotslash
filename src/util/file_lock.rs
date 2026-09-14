/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

//! Wrapper around `std::fs::File::lock`.

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

    #[cfg_attr(
        not(dotslash_internal),
        expect(
            dead_code,
            reason = "only constructed by `FileLock::acquire_shared_lock`"
        )
    )]
    #[error("failed to get shared lock `{0}`")]
    LockShared(PathBuf, #[source] io::Error),
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

            lock_file
                .lock()
                .map_err(|e| FileLockError::LockExclusive(path.to_path_buf(), e))?;

            Ok(FileLock {
                file: Some(lock_file),
            })
        }
        inner(path.as_ref())
    }

    #[cfg_attr(
        not(dotslash_internal),
        expect(dead_code, reason = "only used by the internal release")
    )]
    pub fn acquire_shared_lock<P>(path: P) -> Result<FileLock, FileLockError>
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

            lock_file
                .lock_shared()
                .map_err(|e| FileLockError::LockShared(path.to_path_buf(), e))?;

            Ok(FileLock {
                file: Some(lock_file),
            })
        }
        inner(path.as_ref())
    }

    /// Like [`acquire`](Self::acquire), but returns `Ok(None)` if the lock is
    /// already held instead of blocking.
    pub fn try_acquire<P>(path: P) -> Result<Option<FileLock>, FileLockError>
    where
        P: AsRef<Path>,
    {
        fn inner(path: &Path) -> Result<Option<FileLock>, FileLockError> {
            let lock_file = File::options()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(path)
                .map_err(|e| FileLockError::Create(path.to_path_buf(), e))?;

            match fs2::FileExt::try_lock_exclusive(&lock_file) {
                Ok(()) => Ok(Some(FileLock {
                    file: Some(lock_file),
                })),
                Err(e) if is_lock_contended(&e) => Ok(None),
                Err(e) => Err(FileLockError::LockExclusive(path.to_path_buf(), e)),
            }
        }
        inner(path.as_ref())
    }
}

fn is_lock_contended(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
        || err.raw_os_error() == fs2::lock_contended_error().raw_os_error()
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            drop(file.unlock());
        }
    }
}
