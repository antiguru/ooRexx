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
    target_dir: Option<&std::ffi::OsStr>,
    cwd: &Path,
    exe: &Path,
    manifest: &Path,
) -> [PathBuf; 2] {
    let target = if let Some(target) = target_dir {
        cwd.join(target)
    } else {
        let parent = exe.parent();
        let profile = parent.and_then(|path| {
            if path.file_name().is_some_and(|name| name == "deps") {
                path.parent()
            } else {
                Some(path)
            }
        });
        profile
            .filter(|path| {
                path.file_name()
                    .is_some_and(|name| name == "release" || name == "debug")
            })
            .and_then(Path::parent)
            .map_or_else(|| manifest.join("../../target"), Path::to_path_buf)
    };
    [target.join("release/rexx"), target.join("debug/rexx")]
}

#[cfg(test)]
mod tests;
