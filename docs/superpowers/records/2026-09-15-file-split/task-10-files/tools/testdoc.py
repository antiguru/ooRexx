"""rustdoc over an integration-test target, which `cargo doc` never
documents and `cargo rustdoc --test` cannot (it does not link the library:
measured at BASE, `cargo rustdoc -p rexx-exec --test gate_table_c` fails
E0432 on `rexx_exec`). Replays the target's own rustc invocation (from
`cargo test --no-run -v`, RUSTC_CMD) as rustdoc: the source file, edition,
crate name, `--cfg`/`--check-cfg`, `-L` and `--extern` flags kept; codegen,
emit, JSON and lint flags dropped; `--test` (a harness flag to rustc, a
doctest flag to rustdoc) replaced by `--cfg test`; private items documented.
Prints rustdoc's exit status and its warnings.

usage: testdoc.py RUSTC_CMD_FILE RUST_DIR OUTDIR CARGO_MANIFEST_DIR
"""
import shlex, subprocess, sys, os, re
cmd_file, rust_dir, out, manifest = sys.argv[1:5]
line = open(cmd_file).read()
line = line[line.index("`") + 1 : line.rindex("`")]
args = shlex.split(line)[1:]
keep, i = [], 0
while i < len(args):
    a = args[i]
    if a in ("--cfg", "--check-cfg", "-L", "--extern", "--crate-name", "--edition", "--cap-lints"):
        keep += [a, args[i + 1]]; i += 2; continue
    if a in ("-C", "--out-dir", "--crate-type", "--emit"):
        i += 2; continue
    if a.startswith("--edition=") or a.endswith(".rs"):
        keep.append(a); i += 1; continue
    if a == "--test":
        keep += ["--cfg", "test"]; i += 1; continue
    i += 1
os.makedirs(out, exist_ok=True)
rustdoc = os.path.join(os.path.dirname(shlex.split(line)[0]), "rustdoc")
r = subprocess.run([rustdoc, *keep, "--document-private-items", "-o", out], cwd=rust_dir, env={**os.environ, "CARGO_MANIFEST_DIR": manifest}, capture_output=True, text=True)
print(f"rustdoc exit {r.returncode}")
print(f"warning lines: {len([l for l in r.stderr.splitlines() if l.startswith('warning')])}")
print(r.stderr)
