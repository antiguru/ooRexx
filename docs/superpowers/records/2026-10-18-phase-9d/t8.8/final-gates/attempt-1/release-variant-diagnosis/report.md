# Release CLI build variant diagnosis

The mismatch is caused by the development dependency feature `rustix/pty`. The authorized normal production build with only that feature enabled selected the exact Cargo-test CLI bytes, SHA256 `9c3d09f9f9e39810985b1200007035488a3eba9d00ab6ee7382b6d7aea1b4d06`. The measured normal production pin remains SHA256 `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`. Initial Cargo-zero/verifier-one evidence, approved driver/config, source, HEAD and index remain unchanged under the active freeze.

## Fingerprint trace

`fingerprints-before.json` retains the complete compared records. Normal and test CLI records have equal rustc, profile, rustflags, config, target, path and package features, differing only at `rexx_exec` and `rustix` dependency edges. The execution-library records differ only at `rustix`, whose only extra feature is `pty` and whose only changed dependency is its corresponding build script. All remaining dependency fingerprints agree, while the build-script outputs are byte-identical, SHA256 `7d847dfd34d215f2ce35bd48a35e35a5430fa50280de8a9b13dad50e39817064`.

| Node | Normal fingerprint directory | Test fingerprint directory |
| --- | --- | --- |
| CLI | `rexx-exec-095e073ed0524874` | `rexx-exec-3d4cd710d9320d42` |
| Execution library | `rexx-exec-a64a8b3482d6d7e6` | `rexx-exec-315c6b8e46b965db` |
| Rustix library | `rustix-1c52c0ec65a2485b` | `rustix-1a9d1be0d5de2fde` |
| Rustix build script | `rustix-1ec7bef65744d44f` | `rustix-87448afb0afefcb7` |

The rustix build-script fingerprints have equal compiler/profile/flags/config/target and no dependencies, differing only by `pty`. Both library records retain the same `linux_raw_sys` and `bitflags` dependencies. Fingerprint-local dep-info paths differ with their artifact directories and do not represent another source or compiler option. These concrete differences support a single-feature hypothesis rather than an environment or compiler change.

## Primary documentation and source

Installed Cargo 1.99 documentation under `share/doc/rust/html/cargo/reference/resolver.html#feature-resolver-version-2` describes development-feature unification when building tests. The workspace declares resolver 3, and `rust/crates/rexx-exec/Cargo.toml` enables rustix `pty` under `[dev-dependencies]`, while its normal dependency already enables `fs`. The normal pin's command builds only `--release --bin rexx`, whereas the failed integration build activates that development graph. The one-variable experiment tests this specific documented explanation without changing either declaration.

Pinned source is `/home/moritz/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustix-1.1.4/`, matching the actual Cargo artifact. Its manifest defines `pty = ["fs"]`, and `src/lib.rs:272` exposes the PTY module under that feature, with matching backend-module gates, a Linux raw `OpenptFlags` conversion and the `TIOCGPTPEER` import. There is no additional PTY-conditional branch in an existing runtime API implementation, and `fs` was already enabled in the normal graph. `build.rs` does not use PTY to select another backend or capability path, consistent with the identical actual build outputs.

`source-and-build-output-evidence.json` retains hashes for the relevant manifest, build script, public PTY module and conditional-gate source files. This identifies source-level feature effects without claiming that differently linked binaries are identical or that an unrun functional suite passed. Added feature-gated surface changes Cargo crate identities and can change linked bytes without selecting another existing API source branch. The measured production CLI and Cargo-test CLI therefore remain separately named artifacts.

## One-variable experiment

The owned `/tmp/codex-p9d-t88-diagnose-release-variant.py` loaded only approved-driver definitions, verified its exact hash, and checked the existing source/index freeze around the command. It preserved the test CLI at `retained-test-built-rexx`, outside Cargo output paths. Exactly one normal production build command ran with the same sanitized environment, profile, target, locked dependencies and jobs, adding `--features rustix/pty`; JSON capture changes no codegen setting. No test target, baseline rebuild, reprofile, source edit, output-path binary copy or guard adjustment occurred.

```text
memcap 8G python3 /tmp/codex-p9d-t88-diagnose-release-variant.py pty-build
  cargo build -p rexx-exec -j4 --release --bin rexx --locked --features rustix/pty --message-format=json
```

The child and diagnostic action exited zero, with the child taking 0.1648 seconds. Cargo reported fresh CLI and rustix artifacts, demonstrating that this production selection resolves to the existing test graph without forcing unnecessary recompilation. The resulting CLI matches the retained test CLI by SHA256, while the immutable normal pin is unchanged. `one-variable-result.json` retains profiles/features/executable paths, and separate `diagnosis-rustix-pty-build.stdout`, `.stderr` and `.json` at the final-gate record root retain the exact command and status.

## Smallest validation reconciliation

Keep the measured normal production pin, its hash, source/build provenance, compiler/profile checks and freeze checks unchanged. Record an explicit Cargo-test variant descriptor for the proven PTY graph and `9c3d09f9...` hash, linked to this experiment and the same frozen Rust tree. Release validation can require that exact test-variant CLI and resolved rustix features while checking the normal performance pin independently, with the in-process corpus library/harness retained as separate artifact evidence. This reconciles the verified build variants without pretending that Cargo's test graph is the measured production graph.

No success marker was fabricated, and no approved driver or config value changed. Root must authorize any artifact-guard reconciliation before corpus/suite execution resumes. This is an infrastructure finding, with no product semantic departure or performance-budget ruling. There is no live diagnostic command, and the candidate freeze remains active.
