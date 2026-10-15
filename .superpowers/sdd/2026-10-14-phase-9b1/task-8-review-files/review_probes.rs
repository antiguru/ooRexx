use std::fs;
use rexx_exec::{run_program, run_program_collect_every_alloc};

#[test]
fn probes() {
    let dir = std::env::var("PROBE_DIR").expect("PROBE_DIR");
    let mut names: Vec<_> = fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "rex")).collect();
    names.sort();
    let stress = std::env::var("PROBE_PLAIN").is_err();
    for path in names {
        let text = fs::read(&path).unwrap();
        let inv = rexx_exec::Invocation::none().with_directory(path.parent().unwrap().to_path_buf());
        let p = path.to_str().unwrap();
        let out = if stress { run_program_collect_every_alloc(p, text, inv) } else { run_program(p, text, inv) };
        println!("=== {} rc={} collections={}\n--stdout\n{}--stderr\n{}", path.file_name().unwrap().to_string_lossy(), out.exit_code, out.collections, String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    }
}
