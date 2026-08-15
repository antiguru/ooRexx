### Task 8: adopt timely-dataflow's clippy lint set

Requested by Moritz on 2026-08-15. Source: `https://github.com/TimelyDataflow/timely-dataflow`,
`Cargo.toml`, its `[workspace.lints.clippy]` section. **Re-fetch it and use what you find**, rather
than trusting the transcription below, which the controller took on 2026-08-15.

#### What the controller already measured, so you do not rediscover it

One `cargo clippy --workspace --all-targets` pass over this tree with 42 of timely's warn-level
lints enabled, counted from `--message-format=json` by lint code:

| lint | violations |
|---|---|
| `clippy::as_conversions` | 574 |
| `clippy::shadow_unrelated` | 380 |
| `clippy::needless_pass_by_ref_mut` | 20 |
| every other lint tested | 0 |

Re-measure before you act. If your numbers differ from these, that is a finding: report it.

#### Decisions already taken, by Moritz on 2026-08-15

* **`as_conversions` is `allow`, with a recorded reason.** Not because the lint is wrong but because
  574 sites in a numeric interpreter each need a truncation-and-sign judgement, and a wrong one is a
  silent behavioural change against a byte-for-byte oracle bar. The comment must say what would
  close it: a cast helper in the shape of Materialize's `CastFrom`/`CastLossy`, which this tree does
  not have. Do not write the helper.
* **`shadow_unrelated` is `allow`, with a recorded reason.** 380 renames across files under active
  differential work is churn, and the comment should say so.
* **`needless_pass_by_ref_mut`'s 20 violations are fixed.** Each is a `&mut` parameter never used
  mutably. Fix them by narrowing the parameter, not by silencing the lint.

#### Steps

1. Fetch timely's `[workspace.lints.clippy]` section and reproduce its full membership: the
   allow-level entries and the warn-level entries. Record in your report anything that has changed
   since the controller's reading.
2. Add it to this workspace's `Cargo.toml` under `[workspace.lints.clippy]`, beside the existing
   `[workspace.lints.rust]` block. **Do not touch `unsafe_code = "forbid"`** or the comment above it:
   that line is the record of which crates have been granted an unsafe exception and relaxing it has
   already been done once by mistake and reverted.
3. Set `as_conversions` and `shadow_unrelated` to `allow` with the reasons above stated at the site.
4. Fix the `needless_pass_by_ref_mut` violations. Each fix is a signature narrowing; report the
   count you fixed and confirm it matches the count you measured.
5. **Establish that the adopted lints are actually in force**, which a green build does not show. For
   at least three lints that currently report zero violations, demonstrate the lint fires: introduce
   the violation in a scratch copy, confirm `cargo clippy --workspace --all-targets -- -D warnings`
   goes red, and revert. A lint set that is configured but not reaching the code reads exactly like
   a clean tree.
6. Gates, each exit status read on its own: `cargo fmt --all --check`, `cargo clippy --workspace
   --all-targets -- -D warnings` **from a clean target directory**, `cargo test --release
   --workspace`, and the corpus gate under `REXX_CORPUS_GATE=1`.
7. Report the final lint membership, the three lints you proved live and how, and the violation
   counts before and after.
