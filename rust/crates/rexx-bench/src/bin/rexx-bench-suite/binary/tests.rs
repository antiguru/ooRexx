use super::*;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt as _;

#[test]
fn explicit_target_directories_do_not_fall_back_to_workspace_artifacts() {
    let cwd = Path::new("/invoke");
    let exe = Path::new("/old/target/release/rexx-bench-suite");
    let manifest = Path::new("/workspace/crates/rexx-bench");
    assert_eq!(rust_binary_candidates_for(Some(OsStr::new("/custom")), cwd, exe, manifest),
        [PathBuf::from("/custom/release/rexx"), PathBuf::from("/custom/debug/rexx")]);
    assert_eq!(rust_binary_candidates_for(Some(OsStr::new("custom")), cwd, exe, manifest),
        [PathBuf::from("/invoke/custom/release/rexx"), PathBuf::from("/invoke/custom/debug/rexx")]);
    let non_utf8 = OsString::from_vec(b"/custom-\xff".to_vec());
    let expected = PathBuf::from(&non_utf8);
    assert_eq!(rust_binary_candidates_for(Some(&non_utf8), cwd, exe, manifest),
        [expected.join("release/rexx"), expected.join("debug/rexx")]);
}

#[test]
fn executable_location_preserves_alternate_targets_and_deps() {
    let manifest = Path::new("/workspace/crates/rexx-bench");
    for exe in ["/custom/release/rexx-bench-suite", "/custom/debug/deps/rexx-bench-suite-hash"] {
        assert_eq!(rust_binary_candidates_for(None, Path::new("/invoke"), Path::new(exe), manifest),
            [PathBuf::from("/custom/release/rexx"), PathBuf::from("/custom/debug/rexx")]);
    }
    assert_eq!(rust_binary_candidates_for(None, Path::new("/invoke"), Path::new("/workspace/target/release/deps/test"), manifest),
        [PathBuf::from("/workspace/target/release/rexx"), PathBuf::from("/workspace/target/debug/rexx")]);
}

#[test]
fn unrecognized_executable_uses_the_workspace_target() {
    assert_eq!(rust_binary_candidates_for(None, Path::new("/invoke"), Path::new("/installed/rexx-bench-suite"), Path::new("/workspace/crates/rexx-bench")),
        [PathBuf::from("/workspace/crates/rexx-bench/../../target/release/rexx"), PathBuf::from("/workspace/crates/rexx-bench/../../target/debug/rexx")]);
}
