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
    let parent = exe.parent();
    let under_deps = parent.is_some_and(|path| path.file_name().is_some_and(|name| name == "deps"));
    let profile = parent.and_then(|path| {
        if under_deps {
            path.parent()
        } else {
            Some(path)
        }
    });
    let executable_target = profile
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name == "release" || name == "debug")
        })
        .and_then(Path::parent);
    let target = match target_dir {
        Some(target) if Path::new(target).is_absolute() => PathBuf::from(target),
        // Cargo runs tests from the package directory, losing the origin
        // against which it resolved a relative target directory.
        Some(_) if under_deps && executable_target.is_some() => executable_target
            .expect("recognized test profile")
            .to_path_buf(),
        Some(target) => cwd.join(target),
        None => executable_target.map_or_else(|| manifest.join("../../target"), Path::to_path_buf),
    };
    [target.join("release/rexx"), target.join("debug/rexx")]
}

#[cfg(test)]
mod tests;
