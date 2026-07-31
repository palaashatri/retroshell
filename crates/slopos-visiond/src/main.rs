//! SLOPOS Vision daemon — the background service that owns the vision models
//! and serves OCR / subject-segmentation jobs to SLOPOS apps.
//!
//! Run under the session supervisor, the daemon loads models once, exposes a
//! local IPC interface, and stays out of the UI's way.

fn main() {
    log::warn!("slopos-visiond is not yet implemented");
}
