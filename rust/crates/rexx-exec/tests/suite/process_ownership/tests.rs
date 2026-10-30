use super::*;
use std::process::{Child, Command, Stdio};

struct Sleeper(Child);

impl Drop for Sleeper {
    fn drop(&mut self) {
        let _ = self.0.kill();
        self.0.wait().expect("reap the owned sleeper");
    }
}

#[test]
fn sleeper_checks_only_its_fixture_directory() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("grandchild-ownership-{}", std::process::id()));
    let owned = root.join("owned");
    let unrelated = root.join("unrelated");
    fs::create_dir_all(&owned).expect("owned cwd");
    fs::create_dir_all(&unrelated).expect("unrelated cwd");
    let sleeper = |cwd: &Path| {
        Sleeper(
            Command::new("sleep")
                .arg("41.5")
                .current_dir(cwd)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("sleep starts"),
        )
    };
    let unrelated_child = sleeper(&unrelated);
    assert!(
        !grandchild_sleeps(&owned),
        "another fixture's sleeper is unrelated"
    );
    let owned_child = sleeper(&owned);
    assert!(
        grandchild_sleeps(&owned),
        "the live owned sleeper must be detected"
    );
    drop(owned_child);
    assert!(!grandchild_sleeps(&owned), "the owned sleeper was reaped");
    drop(unrelated_child);
    fs::remove_dir_all(root).expect("remove scratch directories");
}
