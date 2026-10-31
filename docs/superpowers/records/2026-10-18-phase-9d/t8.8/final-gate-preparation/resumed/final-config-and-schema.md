# Final resumed schema and configuration requirements

The prepared runner validates the actual registration-pin schema, without assuming the prior top-level `source_hashes` field. It verifies adjacent source-proof/production-manifest hashes, complete current production/Cargo entry coverage and file hashes, all changed-input hashes, and the normal Cargo artifact/profile against its hashed build record/output. The new registration pin and measured normal pin must both remain `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`, with the gate Rust tree equal to producer `f65066b04fe1f2355ce64d8be581fa9e79641673`. The old PTY proof retains its old source/runtime attribution, with current producer/source proof/manifest and fresh exact test artifacts recorded separately.

Read-only inspection verified all 300 production/Cargo hashes, eleven repaired-input hashes, both normal pin hashes and the supplied build/source proof digests. Root's latest provisional metadata HEAD is `840ef183a615886238ca3c0dded116c9923ad0a1`, but the final freeze HEAD must follow the review/runner record commits. No resumed config or completion attestation was created. Output remains separate at `/tmp/codex-p9d-t88-final-gates-resume/`, and attempt 1 remains untouched.

The following describes the future `/tmp/codex-p9d-t88-final-resume-config.json`; it is not a written config. Root must replace the clearly marked source value with its final full freeze HEAD. The new completion attestation links that source context and identical normal binary to completed profiling, with no active command and no new performance run. The provenance path accepts absolute paths or paths relative to the checkout root.

```json
{
  "source_commit": "<root-supplied final freeze HEAD>",
  "profile_completion_record": "/tmp/codex-p9d-t88-final-resume-profile-completion.json",
  "release_binary": "/tmp/codex-p9d-t88-pins/8.8-corpus-registration-rexx",
  "release_binary_sha256": "d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145",
  "release_pin_provenance": "docs/superpowers/records/2026-10-18-phase-9d/t8.8/corpus-registration-repair/implementation/release-pin.json"
}
```

Driver SHA256 is `898d73dea60af0dea841dca2bb1a6203e9fac9bc6e05fc062572f72392dc6206`, and the separate delta-helper SHA256 is `058a7adb629464b79e2117e57528d4045ae73d032b7bd3d039b31b7dc5b773a5`. `driver.diff` compares against untouched attempt 1, while `actual-schema-driver.diff` isolates this actual-schema correction. The fixed suite-input manifests and whitelist correction remain described in `plan.md`, with one fresh full suite and no reuse exception. No config/freeze/build/test/profile action or tracked/Git/index mutation occurred during preparation.
