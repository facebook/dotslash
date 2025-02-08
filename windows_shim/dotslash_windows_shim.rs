/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
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
#![cfg_attr(feature = "no_std", windows_subsystem = "console")] // Set Entrypoint to "mainCRTStartup"

extern crate alloc;

#[allow(clippy::upper_case_acronyms)]
type DWORD = u32;

use alloc::alloc::GlobalAlloc;
use alloc::alloc::Layout;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::mem;
use core::ptr;
use core::str;

use windows_sys::core::PCWSTR;
use windows_sys::core::PWSTR;
use windows_sys::w;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::BOOL;
use windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::Foundation::TRUE;
use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::Globalization::lstrcatW;
use windows_sys::Win32::Globalization::lstrlenW;
use windows_sys::Win32::Storage::FileSystem::WriteFile;
use windows_sys::Win32::System::Console::GetStdHandle;
use windows_sys::Win32::System::Console::STD_ERROR_HANDLE;
use windows_sys::Win32::System::Environment::GetCommandLineW;
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Memory::GetProcessHeap;
use windows_sys::Win32::System::Memory::HeapAlloc;
use windows_sys::Win32::System::Memory::HeapFree;
use windows_sys::Win32::System::Memory::HeapReAlloc;
use windows_sys::Win32::System::Memory::HEAP_ZERO_MEMORY;
use windows_sys::Win32::System::Threading::CreateProcessW;
use windows_sys::Win32::System::Threading::ExitProcess;
use windows_sys::Win32::System::Threading::GetExitCodeProcess;
use windows_sys::Win32::System::Threading::WaitForSingleObject;
use windows_sys::Win32::System::Threading::INFINITE;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::STARTUPINFOW;
use windows_sys::Win32::UI::Shell::PathCchRemoveExtension;
use windows_sys::Win32::UI::Shell::PathGetArgsW;
use windows_sys::Win32::UI::Shell::PathQuoteSpacesW;

fn write_stderr(text: &str) -> BOOL {
    // lpNumberOfBytesWritten can be NULL only when the lpOverlapped
    // parameter is not NULL.
    let mut bytes_written: u32 = 0;
    unsafe {
        let stdout: HANDLE = GetStdHandle(STD_ERROR_HANDLE);
        let ok: BOOL = WriteFile(
            stdout,                         /* hFile */
            text.as_ptr(),                  /* lpBuffer */
            text.len() as u32,              /* nNumberOfBytesToWrite */
            &mut bytes_written as *mut u32, /* lpNumberOfBytesWritten */
            ptr::null_mut(),                /* lpOverlapped */
        );
        ok
    }
}

fn fatal(text: &str) -> ! {
    write_stderr("dotslash-windows-shim: ");
    write_stderr(text);
    write_stderr("\n");
    unsafe { ExitProcess(1) }
}

fn main_impl() -> ! {
    // CreateProcessW's lpCommandLine has a maximum length of 32,767
    // characters. Allocate that upfront to avoid growing buffers.
    let mut ds_cmd: Vec<u16> = Vec::with_capacity(32767);

    // Start constructing the DotSlash command string.
    ds_cmd.push(0);

    // Append "dotslash " to the command string.
    if unsafe { lstrcatW(ds_cmd.as_mut_ptr(), w!("dotslash ")) }.is_null() {
        fatal("could not append dotslash command string.");
    }

    // Append the DotSlash file path to the command string.
    //
    // NOTE: Turning this executable's full path into a valid argument on the
    // command string will happen in-place in the command string.
    unsafe {
        let ds_file_offset = lstrlenW(ds_cmd.as_ptr()) as usize;
        let ds_file_ptr = ds_cmd.as_mut_ptr().add(ds_file_offset);

        // Get a handle to this executable.
        //
        // When passed NULL, GetModuleHandle returns a handle to the file
        // used to create the calling process (.exe file).
        let handle: HMODULE = GetModuleHandleW(ptr::null_mut());

        // Append the fully qualified path for this executable to the
        // command string.
        //
        // For an executable named `foo.exe` we should now have a command
        // string that looks like `dotslash C:\path\to\foo.exe`.
        //
        // GetModuleFileName requires you to keep growing a buffer until it
        // fits the path. No need to do this because the command buffer
        // is already as large as can be.
        let new_len = GetModuleFileNameW(
            handle,                                    /* hModule */
            ds_file_ptr,                               /* lpFilename */
            (ds_cmd.capacity() - ds_file_offset) as _, /* nSize */
        ) as usize;
        if new_len == 0 {
            fatal("GetModuleFileNameW failed.");
        }

        // Remove the extension from this executable's full path.
        //
        // We assume that the DotSlash file is named just like this
        // executable but without the `.exe`.
        //
        // For an executable named `foo.exe` we should now have a command
        // string that looks like `dotslash C:\path\to\foo`.
        //
        // PathCchRemoveExtension returns `!= S_OK` when there is no extension.
        // In this case, we'll pass this executable as the DotSlash file path.
        // `dotslash` will fail and complain that it's not a valid DotSlash
        // file.
        PathCchRemoveExtension(ds_file_ptr, new_len + 1);

        // Quote the entire DotSlash file path if there are spaces.
        //
        // No need to worry about escaping quotes because those aren't
        // allowed in Windows paths.
        //
        // For an executable named `foo.exe` this is a noop.
        //
        // For an executable named `foo bar.exe` we should now have a command
        // string that looks like `dotslash "C:\path\to\foo bar"`.
        PathQuoteSpacesW(ds_file_ptr);

        // Get the arguments that were passed to us.
        let line_ptr: PCWSTR = GetCommandLineW();
        // Skip `argv[0]` and focus on the remaining arguments.
        let args_ptr: PWSTR = PathGetArgsW(line_ptr);
        // Append the arguments to the command string if there are any.
        if *args_ptr != 0 {
            // Append a separator for the arguments.
            if lstrcatW(ds_cmd.as_mut_ptr(), w!(" ")).is_null() {
                fatal("could not append command args separator.");
            }
            lstrcatW(ds_cmd.as_mut_ptr(), args_ptr);
        }
    }

    // Run the command string.

    let mut si: STARTUPINFOW = unsafe { mem::zeroed() };
    si.cb = mem::size_of::<STARTUPINFOW>() as DWORD;
    let mut pi: PROCESS_INFORMATION = unsafe { mem::zeroed() };

    let status = unsafe {
        CreateProcessW(
            ptr::null_mut(),     // lpApplicationName
            ds_cmd.as_mut_ptr(), // lpCommandLine
            ptr::null_mut(),     // lpProcessAttributes
            ptr::null_mut(),     // lpThreadAttributes
            TRUE,                // bInheritHandles
            0,                   // dwCreationFlags
            ptr::null_mut(),     // lpEnvironment
            ptr::null_mut(),     // lpCurrentDirectory
            &si,
            &mut pi,
        )
    };

    // Once CreateProcessW is called, there is no need to hold onto
    // lpCommandLine.
    // https://stackoverflow.com/a/31031165
    drop(ds_cmd);

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

        unsafe {
            CloseHandle(pi.hProcess);
            CloseHandle(pi.hThread);
            ExitProcess(status)
        };
    }

    let err = unsafe { GetLastError() };
    if err == ERROR_FILE_NOT_FOUND {
        fatal("dotslash executable not found.");
    }

    fatal("could not execute dotslash command.");
}

#[no_mangle]
#[cfg(feature = "no_std")]
pub extern "C" fn mainCRTStartup() -> ! {
    main_impl()
}

#[cfg(not(feature = "no_std"))]
fn main() -> ! {
    main_impl()
}

//
// Allocator and abort handling
//

// https://github.com/rust-lang/rust/issues/62785
#[used]
#[no_mangle]
#[cfg(feature = "no_std")]
pub static _fltused: i32 = 0;

#[panic_handler]
#[cfg(feature = "no_std")]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // https://github.com/rust-lang/rust/blob/1.75.0/library/panic_abort/src/lib.rs#L58-L83
    #[cfg(windows)]
    unsafe {
        // https://learn.microsoft.com/en-us/cpp/intrinsics/fastfail
        const FAST_FAIL_FATAL_APP_EXIT: usize = 7;
        cfg_if::cfg_if! {
            if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
                core::arch::asm!("int $$0x29", in("ecx") FAST_FAIL_FATAL_APP_EXIT);
                core::intrinsics::unreachable();
            } else if #[cfg(target_arch = "arm", target_feature = "thumb-mode")] {
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
#[no_mangle]
#[cfg(feature = "no_std")]
pub extern "C" fn eh_personality() {}

struct Win32HeapAlloc;

// Based on https://github.com/rust-lang/rust/blob/1.75.0/compiler/rustc_codegen_cranelift/example/alloc_system.rs#L70-L124
unsafe impl GlobalAlloc for Win32HeapAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HeapAlloc(GetProcessHeap(), 0, layout.size()) as *mut u8
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        HeapFree(GetProcessHeap(), 0, ptr as *mut c_void);
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ptr as *mut c_void,
            new_size,
        ) as *mut u8
    }
}

#[global_allocator]
static GLOBAL: Win32HeapAlloc = Win32HeapAlloc;
