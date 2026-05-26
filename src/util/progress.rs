/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

/// A bit less than 80 chars so it fits on standard terminals.
const NUM_PROGRESS_BAR_CHARS: u8 = 70;

#[must_use]
pub fn display_progress(content_length: u64, output_path: &Path) -> (Sender<()>, JoinHandle<()>) {
    let path = output_path.to_path_buf();

    display_progress_with_size(content_length, move || {
        fs::metadata(&path).ok().map(|attr| attr.len())
    })
}

#[must_use]
pub fn display_progress_with_size<F>(
    content_length: u64,
    mut current_size: F,
) -> (Sender<()>, JoinHandle<()>)
where
    F: FnMut() -> Option<u64> + Send + 'static,
{
    // Channel to inform the progress thread that the download has finished
    // early. This can be because of an error (`send` is dropped) or because
    // the `content_length` is incorrect (`send` sends `()`).
    let (send, recv) = mpsc::channel();

    let handle = thread::spawn(move || {
        // This is the progress against NUM_PROGRESS_BAR_CHARS.
        let mut last_progress: u8 = 0;
        eprint!("[{}]", " ".repeat(NUM_PROGRESS_BAR_CHARS as usize));

        // Poll quickly for the download to start
        let short_pause = Duration::from_millis(10);
        loop {
            if should_end_progress(&recv) {
                return;
            }
            if current_size().is_some() {
                break;
            }
            // Download hasn't started: pause and try again.
            thread::sleep(short_pause);
        }

        let pause = Duration::from_millis(100);
        loop {
            let size = current_size().unwrap_or(0);
            let is_complete = size >= content_length;
            let delta = if is_complete || content_length == 0 {
                NUM_PROGRESS_BAR_CHARS - last_progress
            } else {
                let current_progress = (f64::from(NUM_PROGRESS_BAR_CHARS)
                    * (size as f64 / content_length as f64))
                    as u8;
                let delta = current_progress - last_progress;
                last_progress = current_progress;
                delta
            };
            if delta != 0 && last_progress > 0 {
                let num_equals = last_progress - 1;
                let num_space = NUM_PROGRESS_BAR_CHARS - last_progress;
                // Admittedly, this is not the most efficient way to animate
                // the progress bar, but it is simple so that it works
                // cross-platform without pulling in a more heavyweight crate
                // for dealing with ANSI escape codes.
                eprint!(
                    "\r[{}>{}]",
                    "=".repeat(num_equals as usize),
                    " ".repeat(num_space as usize)
                );
            }

            if is_complete || should_end_progress(&recv) {
                eprintln!("\r[{}]", "=".repeat(NUM_PROGRESS_BAR_CHARS as usize));
                break;
            }

            thread::sleep(pause);
        }
    });

    (send, handle)
}

fn should_end_progress(recv: &Receiver<()>) -> bool {
    match recv.try_recv() {
        Ok(()) | Err(TryRecvError::Disconnected) => true,
        Err(TryRecvError::Empty) => false,
    }
}
