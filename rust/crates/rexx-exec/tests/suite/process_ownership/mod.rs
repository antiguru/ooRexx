use std::fs;
use std::path::Path;

/// Whether a process runs `sleep 41.5`, which `Grandchild`'s test starts.
pub(super) fn grandchild_sleeps(run: &Path) -> bool {
    let run = run.canonicalize().expect("canonical fixture run directory");
    fs::read_dir("/proc")
        .expect("/proc")
        .filter_map(Result::ok)
        .any(|entry| {
            fs::read(entry.path().join("cmdline"))
                .is_ok_and(|cmdline| cmdline == b"sleep\x0041.5\x00")
                && fs::read_link(entry.path().join("cwd")).is_ok_and(|cwd| cwd == run)
        })
}

#[cfg(test)]
mod tests;
