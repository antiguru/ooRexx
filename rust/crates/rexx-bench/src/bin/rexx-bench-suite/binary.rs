use std::path::{Path, PathBuf};

/// Where `rexx` is built, release first.
pub(super) fn rust_binary_candidates() -> [PathBuf; 2] {
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    [target.join("release/rexx"), target.join("debug/rexx")]
}

