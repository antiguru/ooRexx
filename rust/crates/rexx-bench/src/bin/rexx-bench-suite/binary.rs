use std::path::{Path, PathBuf};

/// Where `rexx` is built, release first.
pub(super) fn rust_binary_candidates() -> [PathBuf; 2] {
    rust_binary_candidates_for(
        std::env::var_os("CARGO_TARGET_DIR").as_deref(),
        &std::env::current_dir().expect("invocation cwd"),
        &std::env::current_exe().expect("benchmark executable"),
        Path::new(env!("CARGO_MANIFEST_DIR")),
    )
}

fn rust_binary_candidates_for(
    _target_dir: Option<&std::ffi::OsStr>,
    _cwd: &Path,
    _exe: &Path,
    _manifest: &Path,
) -> [PathBuf; 2] {
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    [target.join("release/rexx"), target.join("debug/rexx")]
}


#[cfg(test)]
mod tests;
