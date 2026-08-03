/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

//! The "DotSlash Windows Shim" is a tiny executable (.exe) to workaround
//! Windows' lack of shebang support. The shim is placed next to a DotSlash
//! file (with the DotSlash filename plus `.exe`) and it runs the DotSlash
//! file and forwards all arguments and IO streams.

//! This code is optimized for size since the shim is meant to be checked
//! into source control for every DotSlash file that needs to be run on
//! Windows. Only Windows APIs are used to avoid increasing binary size.

#![cfg_attr(feature = "no_std", allow(internal_features))]
#![cfg_attr(feature = "no_std", feature(core_intrinsics))]
#![cfg_attr(feature = "no_std", feature(lang_items))]
#![cfg_attr(feature = "no_std", no_std)]
#![cfg_attr(feature = "no_std", no_main)]
// Select the console subsystem; the no_std entry point is `mainCRTStartup` below.
#![cfg_attr(feature = "no_std", windows_subsystem = "console")]

#[allow(clippy::upper_case_acronyms)]
type DWORD = u32;

use core::mem;
use core::ptr;
use core::str;

use windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::TRUE;
use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::Storage::FileSystem::WriteFile;
use windows_sys::Win32::System::Console::GetStdHandle;
use windows_sys::Win32::System::Console::STD_ERROR_HANDLE;
use windows_sys::Win32::System::Environment::GetCommandLineW;
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows_sys::Win32::System::Memory::GetProcessHeap;
use windows_sys::Win32::System::Memory::HeapAlloc;
use windows_sys::Win32::System::Memory::HeapFree;
use windows_sys::Win32::System::Threading::CreateProcessW;
use windows_sys::Win32::System::Threading::ExitProcess;
use windows_sys::Win32::System::Threading::GetExitCodeProcess;
use windows_sys::Win32::System::Threading::INFINITE;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::STARTUPINFOW;
use windows_sys::Win32::System::Threading::WaitForSingleObject;
use windows_sys::core::BOOL;
use windows_sys::core::PCWSTR;
use windows_sys::w;

fn write_stderr(text: &str) -> BOOL {
    // lpNumberOfBytesWritten can be NULL only when the lpOverlapped
    // parameter is not NULL.
    let mut bytes_written: u32 = 0;
    unsafe {
        let stderr: HANDLE = GetStdHandle(STD_ERROR_HANDLE);
        let ok: BOOL = WriteFile(
            stderr,                         /* hFile */
            text.as_ptr(),                  /* lpBuffer */
            text.len() as u32,              /* nNumberOfBytesToWrite */
            &mut bytes_written as *mut u32, /* lpNumberOfBytesWritten */
            ptr::null_mut(),                /* lpOverlapped */
        );
        ok
    }
}

fn fatal(text: &str) -> ! {
    // Diagnostics are best-effort because there is no useful recovery if
    // stderr itself cannot be written.
    write_stderr("dotslash-windows-shim: ");
    write_stderr(text);
    write_stderr("\n");
    unsafe { ExitProcess(1) }
}

// Find the raw argument tail without parsing and reconstructing it, so the
// caller's quoting and backslashes reach dotslash unchanged. Only argv[0] is
// scanned: quotes may delimit spans anywhere within it, while spaces and tabs
// terminate it only when outside quotes.
//
// SAFETY: `p` must point to a readable, null-terminated UTF-16 string.
unsafe fn command_line_args(mut p: *const u16) -> *const u16 {
    let mut in_quotes = false;

    loop {
        let ch = unsafe { *p };

        if ch == 0 {
            return p;
        }

        if ch == b'"' as u16 {
            in_quotes = !in_quotes;
        } else if !in_quotes && (ch == b' ' as u16 || ch == b'\t' as u16) {
            break;
        }

        p = unsafe { p.add(1) };
    }

    while unsafe { *p } == b' ' as u16 || unsafe { *p } == b'\t' as u16 {
        p = unsafe { p.add(1) };
    }

    p
}

// CreateProcessW's lpCommandLine has a maximum length of 32,767 UTF-16 code
// units, including its terminating null.
const BUF_MAX_SIZE: usize = 32767;

struct PoorMansString {
    buf: *mut u16,
    len: usize,
}

impl PoorMansString {
    fn append(&mut self, mut other: *const u16) {
        while unsafe { *other } != 0 {
            if self.len == self.capacity() {
                fatal("Buffer overflow");
            }
            unsafe {
                *self.buf.add(self.len) = *other;
                other = other.add(1);
            }
            self.len += 1;
        }
        unsafe { *self.buf.add(self.len) = 0 };
    }

    fn capacity(&self) -> usize {
        // append() writes a null after every copy, so one code unit is never
        // available for content.
        BUF_MAX_SIZE - 1
    }

    fn new() -> Self {
        // Allocate once at the API's maximum size. Zero-initialization is
        // unnecessary because append() writes both content and its terminator.
        let buf = unsafe {
            HeapAlloc(GetProcessHeap(), 0, BUF_MAX_SIZE * mem::size_of::<u16>()) as *mut u16
        };
        Self { buf, len: 0 }
    }
}

fn main_impl() -> ! {
    let mut ds_cmd = PoorMansString::new();

    // Append `dotslash "` to the command string.
    ds_cmd.append(w!("dotslash \""));

    // Append the DotSlash file path to the command string.
    //
    // NOTE: Turning this executable's full path into a valid argument on the
    // command string will happen in-place in the command string.
    unsafe {
        let ds_file_ptr = ds_cmd.buf.add(ds_cmd.len);
        // Append the fully qualified path for this executable to the
        // command string.
        //
        // For an executable named `foo.exe` we should now have a command
        // string that looks like `dotslash "C:\path\to\foo.exe`.
        //
        // GetModuleFileNameW reports truncation when its buffer is too small.
        // Retrying with a larger allocation cannot help because the completed
        // CreateProcessW command must fit in this same maximum-sized buffer.
        let remaining_capacity = ds_cmd.capacity() - ds_cmd.len;
        let new_len = GetModuleFileNameW(
            ptr::null_mut(),         /* hModule */
            ds_file_ptr,             /* lpFilename */
            remaining_capacity as _, /* nSize */
        ) as usize;
        if new_len == 0 || new_len == remaining_capacity {
            fatal("GetModuleFileNameW failed.");
        }
        ds_cmd.len += new_len;

        // Remove the final `.exe` extension from this executable's full path.
        //
        // The shim contract requires it to be named `<DotSlash-file>.exe`, so
        // the suffix is unconditionally removed as four UTF-16 code units. No
        // case check is needed, which also handles the conventional `.EXE`.
        //
        // The command now looks like `dotslash "C:\path\to\foo`.
        //
        ds_cmd.len -= 4;

        // Always close the quote around the DotSlash file path. Quoting paths
        // without spaces is valid, and no escaping is needed because quotes
        // are not allowed in Windows paths.
        ds_cmd.append(w!("\""));

        // Get the arguments that were passed to us.
        let line_ptr: PCWSTR = GetCommandLineW();
        // Skip `argv[0]` and focus on the remaining arguments.
        let args_ptr = command_line_args(line_ptr);
        // Append the arguments to the command string if there are any.
        if *args_ptr != 0 {
            // Normalize the discarded argv[0] separator whitespace to one
            // space. The argument tail itself remains untouched.
            ds_cmd.append(w!(" "));
            ds_cmd.append(args_ptr);
        }
    }

    // Run the command string.

    let mut si: STARTUPINFOW = unsafe { mem::zeroed() };
    si.cb = mem::size_of::<STARTUPINFOW>() as DWORD;
    let mut pi: PROCESS_INFORMATION = unsafe { mem::zeroed() };

    // A null application name makes CreateProcessW resolve the first command
    // token (`dotslash`) normally, including through PATH. Handles, the
    // environment, and the working directory are inherited so the shim is
    // transparent to the child process.
    let status = unsafe {
        CreateProcessW(
            ptr::null_mut(), // lpApplicationName
            ds_cmd.buf,      // lpCommandLine
            ptr::null_mut(), // lpProcessAttributes
            ptr::null_mut(), // lpThreadAttributes
            TRUE,            // bInheritHandles
            0,               // dwCreationFlags
            ptr::null_mut(), // lpEnvironment
            ptr::null_mut(), // lpCurrentDirectory
            &si,
            &mut pi,
        )
    };

    // CreateProcessW has finished reading the command line when it returns, so
    // release the maximum-sized buffer before a potentially long child wait.
    // Capture a failure code first because cleanup may change the thread's
    // last-error value.
    let err = unsafe { GetLastError() };
    unsafe { HeapFree(GetProcessHeap(), 0, ds_cmd.buf.cast()) };

    if status == TRUE {
        let res = unsafe { WaitForSingleObject(pi.hProcess, INFINITE) };
        if res != WAIT_OBJECT_0 {
            fatal("WaitForSingleObject failed.");
        }

        let mut status = 0;
        let res = unsafe { GetExitCodeProcess(pi.hProcess, &mut status) };
        if res != TRUE {
            fatal("could not get dotslash command exit code.");
        }

        // Process teardown closes both handles in pi. Closing the thread handle
        // earlier would add code and an import without materially reducing RSS.
        unsafe { ExitProcess(status) };
    }

    if err == ERROR_FILE_NOT_FOUND {
        fatal("dotslash executable not found.");
    }

    fatal("could not execute dotslash command.");
}

#[cfg(not(feature = "no_std"))]
fn main() -> ! {
    main_impl()
}

// TODO: get rid of this and stop linking to msvcrt
#[cfg(fbcode_build)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    main_impl()
}

#[cfg(all(feature = "no_std", not(fbcode_build)))]
#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> ! {
    main_impl()
}

//
// Abort handling
//

#[panic_handler]
#[cfg(feature = "no_std")]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    // https://github.com/rust-lang/rust/blob/1.75.0/library/panic_abort/src/lib.rs#L58-L83
    #[cfg(windows)]
    unsafe {
        // https://learn.microsoft.com/en-us/cpp/intrinsics/fastfail
        const FAST_FAIL_FATAL_APP_EXIT: usize = 7;
        cfg_if::cfg_if! {
            if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
                core::arch::asm!("int $$0x29", in("ecx") FAST_FAIL_FATAL_APP_EXIT);
                core::intrinsics::unreachable();
            } else if #[cfg(all(target_arch = "arm", target_feature = "thumb-mode"))] {
                core::arch::asm!(".inst 0xDEFB", in("r0") FAST_FAIL_FATAL_APP_EXIT);
                core::intrinsics::unreachable();
            } else if #[cfg(target_arch = "aarch64")] {
                core::arch::asm!("brk 0xF003", in("x0") FAST_FAIL_FATAL_APP_EXIT);
                core::intrinsics::unreachable();
            }
        }
    }

    // For the benefit of check builds on non-Windows.
    #[cfg(not(windows))]
    core::intrinsics::abort();
}

#[lang = "eh_personality"]
#[cfg(feature = "no_std")]
unsafe extern "C" fn rust_eh_personality() {}
