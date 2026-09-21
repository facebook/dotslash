/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use std::io;
use std::io::BufRead;
use std::io::Read;
use std::io::Seek;
use std::path::Path;

use bzip2::read::BzDecoder;
use flate2::bufread::GzDecoder;
#[cfg(not(dotslash_internal))]
use liblzma::bufread::XzDecoder;
use tar::Archive;
#[cfg(not(dotslash_internal))]
use zip::ZipArchive;
use zstd::stream::read::Decoder as ZstdDecoder;

use crate::util::fs_ctx;

#[derive(Copy, Clone)]
pub enum ArchiveType {
    Tar,
    #[cfg(not(dotslash_internal))]
    Bzip2,
    TarBzip2,
    #[cfg(not(dotslash_internal))]
    Gz,
    TarGz,
    #[cfg(not(dotslash_internal))]
    Xz,
    #[cfg(not(dotslash_internal))]
    TarXz,
    #[cfg(not(dotslash_internal))]
    Zstd,
    TarZstd,
    #[cfg(not(dotslash_internal))]
    Zip,
    #[cfg(not(dotslash_internal))]
    Pkg,
}

/// Attempts to extract the tar/zip archive into the specified directory
/// or file.
///
/// To extract tars, this uses the tar crate (https://crates.io/crates/tar)
/// directly. Those who create compressed artifacts for DotSlash are
/// responsible for ensuring they can be decompressed with its version of tar.
pub fn unarchive<R>(reader: R, destination: &Path, archive_type: ArchiveType) -> io::Result<()>
where
    R: BufRead + Seek,
{
    match archive_type {
        ArchiveType::Tar => unpack_tar(reader, destination),

        #[cfg(not(dotslash_internal))]
        ArchiveType::Bzip2 => write_out(BzDecoder::new(reader), destination),
        ArchiveType::TarBzip2 => unpack_tar(BzDecoder::new(reader), destination),

        #[cfg(not(dotslash_internal))]
        ArchiveType::Gz => write_out(GzDecoder::new(reader), destination),
        ArchiveType::TarGz => unpack_tar(GzDecoder::new(reader), destination),

        #[cfg(not(dotslash_internal))]
        ArchiveType::Xz => write_out(XzDecoder::new(reader), destination),
        #[cfg(not(dotslash_internal))]
        ArchiveType::TarXz => unpack_tar(XzDecoder::new(reader), destination),

        #[cfg(not(dotslash_internal))]
        ArchiveType::Zstd => write_out(ZstdDecoder::with_buffer(reader)?, destination),
        ArchiveType::TarZstd => unpack_tar(ZstdDecoder::with_buffer(reader)?, destination),

        #[cfg(not(dotslash_internal))]
        ArchiveType::Zip => {
            let destination = fs_ctx::canonicalize(destination)?;
            let mut archive = ZipArchive::new(reader)?;
            archive.extract(destination)?;
            Ok(())
        }

        #[cfg(not(dotslash_internal))]
        ArchiveType::Pkg => unpack_pkg(reader, destination),
    }
}

#[cfg(not(dotslash_internal))]
fn write_out<R>(mut reader: R, destination_dir: &Path) -> io::Result<()>
where
    R: Read,
{
    let mut output_file = fs_ctx::file_create(destination_dir)?;
    io::copy(&mut reader, &mut output_file)?;
    Ok(())
}

#[cfg(all(not(dotslash_internal), target_os = "macos"))]
fn unpack_pkg<R>(mut reader: R, destination: &Path) -> io::Result<()>
where
    R: BufRead + Seek,
{
    let destination = fs_ctx::canonicalize(destination)?;
    let work = tempfile::tempdir()?;
    let package = work.path().join("artifact.pkg");
    io::copy(&mut reader, &mut fs_ctx::file_create(&package)?)?;
    let expanded = work.path().join("expanded");
    let output = std::process::Command::new("/usr/sbin/pkgutil")
        .arg("--expand-full")
        .arg(&package)
        .arg(&expanded)
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "pkgutil failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Expand payloads without installing the package or executing its scripts.
    let mut pending = vec![expanded];
    let mut payloads = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if entry.file_name() == "Payload" {
                payloads.push(entry.path());
            } else {
                pending.push(entry.path());
            }
        }
    }
    if payloads.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "PKG contains no file payload",
        ));
    }
    payloads.sort();
    for payload in payloads {
        let output = std::process::Command::new("/usr/bin/ditto")
            .arg(payload)
            .arg(&destination)
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other(format!(
                "ditto failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }
    Ok(())
}

#[cfg(all(not(dotslash_internal), not(target_os = "macos")))]
fn unpack_pkg<R>(_reader: R, _destination: &Path) -> io::Result<()>
where
    R: BufRead + Seek,
{
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "PKG extraction is only supported on macOS",
    ))
}

fn unpack_tar<R>(reader: R, destination_dir: &Path) -> io::Result<()>
where
    R: Read,
{
    // The destination dir is canonicalized for the benefit of Windows, but we
    // do it on all platforms for consistency of behavior.
    //
    // Windows has a path length limit of 255 chars. "Extended-length paths"[1]
    // are paths starting with `\\?\`. These are not subject to the length
    // limit, but have other issues: they cannot use forward slashes.
    //
    // `fs::canonicalize` will both prefix the path with `\\?\` and normalize
    // the slashes[2]. This is important because we don't know the depth of the
    // tarball file structure (so we need to avoid possible path length
    // limits), and we don't know if the destination path is mixing slashes.
    //
    // We only use extended-length paths here and not earlier because you
    // can't exec `.bat` files with `\\?\` (although `.exe` files are ok).
    //
    // We canonicalize for all platforms because `fs::canonicalize` can
    // error[3] and not everyone can test on Windows.
    //
    // [1] https://docs.microsoft.com/en-us/windows/desktop/FileIO/naming-a-file#maxpath
    // [2] https://doc.rust-lang.org/std/fs/fn.canonicalize.html#platform-specific-behavior
    // [3] https://doc.rust-lang.org/std/fs/fn.canonicalize.html#errors

    let destination_dir = fs_ctx::canonicalize(destination_dir)?;

    let mut archive = Archive::new(reader);
    archive.set_preserve_permissions(true);
    archive.set_preserve_mtime(true);
    archive.unpack(destination_dir)
}

#[cfg(all(test, not(dotslash_internal), target_os = "macos"))]
mod pkg_tests {
    use super::{ArchiveType, unarchive};
    use std::fs;
    use std::io;
    use std::io::BufReader;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::Path;
    use std::process::Command;

    fn run(command: &mut Command) {
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn build_package(work: &Path, name: &str) -> std::path::PathBuf {
        let root = work.join(format!("{name}-root"));
        fs::create_dir_all(root.join("bin")).unwrap();
        let executable = root.join("bin").join(name);
        fs::write(&executable, b"#!/bin/sh\necho package-ok\n").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        symlink(name, root.join("bin").join(format!("{name}-link"))).unwrap();
        let scripts = work.join(format!("{name}-scripts"));
        fs::create_dir(&scripts).unwrap();
        let script = scripts.join("preinstall");
        fs::write(&script, b"#!/bin/sh\nexit 99\n").unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
        let package = work.join(format!("{name}.pkg"));
        run(Command::new("/usr/bin/pkgbuild")
            .arg("--root")
            .arg(root)
            .arg("--identifier")
            .arg(format!("org.example.{name}"))
            .args(["--version", "1", "--install-location", "/opt/example"])
            .arg("--scripts")
            .arg(scripts)
            .arg(&package));
        package
    }

    #[test]
    fn extracts_component_and_distribution_payloads() {
        let work = tempfile::Builder::new()
            .prefix("dotslash pkg's ")
            .tempdir()
            .unwrap();
        let first = build_package(work.path(), "first");
        let second = build_package(work.path(), "second");
        let distribution = work.path().join("distribution.pkg");
        run(Command::new("/usr/bin/productbuild")
            .arg("--package")
            .arg(&first)
            .arg("--package")
            .arg(second)
            .arg(&distribution));
        for (package, names) in [
            (&first, vec!["first"]),
            (&distribution, vec!["first", "second"]),
        ] {
            let destination = tempfile::Builder::new()
                .prefix("payload's ")
                .tempdir()
                .unwrap();
            unarchive(
                BufReader::new(fs::File::open(package).unwrap()),
                destination.path(),
                ArchiveType::Pkg,
            )
            .unwrap();
            for name in names {
                let executable = destination.path().join("bin").join(name);
                assert_eq!(
                    fs::read(&executable).unwrap(),
                    b"#!/bin/sh\necho package-ok\n"
                );
                assert_ne!(
                    fs::metadata(&executable).unwrap().permissions().mode() & 0o111,
                    0
                );
                assert_eq!(
                    fs::read_link(destination.path().join("bin").join(format!("{name}-link")))
                        .unwrap(),
                    Path::new(name)
                );
            }
            assert!(!destination.path().join("opt").exists());
            assert!(!destination.path().join("Scripts").exists());
        }
    }

    #[test]
    fn rejects_invalid_package() {
        let destination = tempfile::tempdir().unwrap();
        let result = unarchive(
            io::Cursor::new(b"not a package"),
            destination.path(),
            ArchiveType::Pkg,
        );
        assert!(result.unwrap_err().to_string().contains("pkgutil failed"));
    }
}

#[cfg(all(test, not(dotslash_internal), not(target_os = "macos")))]
#[test]
fn pkg_extraction_requires_macos() {
    let error = unarchive(
        io::Cursor::new(b"not a package"),
        Path::new("unused"),
        ArchiveType::Pkg,
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Unsupported);
}
