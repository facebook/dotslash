/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

macro_rules! if_platform {
    (
        linux_aarch64 = $linux_aarch64:tt,
        linux_x86_64 = $linux_x86_64:tt,
        macos_aarch64 = $macos_aarch64:tt,
        macos_x86_64 = $macos_x86_64:tt,
        windows_aarch64 = $windows_aarch64:tt,
        windows_x86_64 = $windows_x86_64:tt,
    ) => {
        if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
            $linux_aarch64
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            $linux_x86_64
        } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            $macos_aarch64
        } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
            $macos_x86_64
        } else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
            $windows_aarch64
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            $windows_x86_64
        } else {
            panic!("unknown arch");
        }
    };
}

pub(crate) use if_platform;

pub const SUPPORTED_PLATFORM: &str = if_platform! {
    linux_aarch64 = "linux-aarch64",
    linux_x86_64 = "linux-x86_64",
    macos_aarch64 = "macos-aarch64",
    macos_x86_64 = "macos-x86_64",
    windows_aarch64 = "windows-aarch64",
    windows_x86_64 = "windows-x86_64",
};
