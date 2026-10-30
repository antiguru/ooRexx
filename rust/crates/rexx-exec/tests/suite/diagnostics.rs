//! Optional per-run evidence retained before the suite removes its copy.

use super::*;

pub(super) fn capture(run: &Path, side: &str, process: &ProcessRun, result: &GroupRun) {
    let Some(root) = std::env::var_os("REXX_SUITE_DIAGNOSTICS") else {
        return;
    };
    let work = run.parent().expect("suite work directory");
    let directory = PathBuf::from(root)
        .join(work.file_name().expect("work name"))
        .join(format!("{side}-{}", result.iterations));
    fs::create_dir_all(&directory).expect("create suite diagnostic directory");
    fs::write(directory.join("stdout"), &process.run.stdout).expect("retain suite stdout");
    fs::write(directory.join("stderr"), &process.run.stderr).expect("retain suite stderr");
    let mut context = format!(
        "run={}\nTZ={:?}\nstatus={:?} ended={:?} took={:?}\n{result:#?}\n",
        run.display(),
        std::env::var_os("TZ"),
        process.run.status,
        process.ended,
        process.took
    );
    for path in [
        marker(run),
        run.join("child.rex"),
        interpreter_bin(run).join("rexx"),
        run.join("ooRexx")
            .join(format!("{}.testGroup", result.group)),
    ] {
        let _ = writeln!(
            context,
            "path={} canonical={:?} metadata={:?}",
            path.display(),
            path.canonicalize(),
            fs::metadata(&path)
        );
        if path
            .file_name()
            .is_some_and(|name| name == "child.rex" || name == "started.txt")
        {
            match fs::read(&path) {
                Ok(bytes) => fs::write(directory.join(path.file_name().unwrap()), bytes)
                    .expect("retain fixture"),
                Err(error) => {
                    let _ = writeln!(context, "read failed: {error}");
                }
            }
        }
    }
    fs::write(directory.join("context.txt"), context).expect("retain suite context");
}
