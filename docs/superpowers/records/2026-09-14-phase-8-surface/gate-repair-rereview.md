# Gate table C repair: fix round 1 re-review

Reviewed `5757857a4..5b0b69170` (commit `5b0b69170`; `ec7824e4a`/`efe70f926` are Moritz's
unrelated plan document, ignored). Read-only on the worktree; a full gate run was active
in its `target/` throughout, so all builds ran from two independent `git archive HEAD`
extractions under this session's scratchpad, each with its own `CARGO_TARGET_DIR`, at
paths unrelated to both this worktree and each other
(`.../rereview/loc-a/nested/deeper`, `.../rereview/loc-b`). Both deleted by explicit path
after use.

## Verdict: APPROVED

The finding closes, the guard generalizes, the mechanism it depends on is confirmed by
reading the actual code path (not inferred from two coincidentally-working locations),
and nothing else these two commits touched carries a machine-specific property. One
non-blocking residual risk noted under item 3.

## Per-item findings

1. **Does the finding close?** Yes, verified independently in two fresh locations built
   from scratch, neither related to this worktree or to each other: ran the committed
   `streamsupplier__instance.rex` in place (each location's own freshly-built `rexx-run`,
   invoked from the probe's own directory, matching how the real harness invokes it)
   against the oracle, three descriptors. Both locations: crate rc=0, oracle rc=0, 8
   lines of stdout byte-identical, stderr identical. Real answers, not a shared refusal.
2. **`no_construction_program_embeds_this_checkouts_own_location`.** `repo_root()`
   (`rexx-extract/tests/extract_docs.rs:21`-`22`) is `env!("CARGO_MANIFEST_DIR").join("../../..")`,
   the same pattern `gate_table_c.rs:30`'s `corpus_dir()` already uses -- computed fresh
   at compile time from wherever that build's crate actually lives, not a literal
   string. Confirmed the guard generalizes, not just for the original path: in one of
   the two fresh locations (root unrelated to `/home/moritz`), I ran the test as
   committed (passed), then edited the pristine copy's own `classes.rs` to hardcode
   *that location's own* absolute path into the STREAMSUPPLIER entry and reran -- it
   failed, naming that path exactly, with the same message shape the implementer
   reports for the original defect. Restored the file afterward; nothing in the
   tracked worktree was touched. I did not additionally reproduce the original failure
   against `/home/moritz/dev/repos/ooRexx-rust-rewrite` itself (building there would
   collide with the live gate run), but the mechanism is now verified as
   location-generic, and `repo_root()` computed from a build rooted at that exact path
   is `/home/moritz/dev/repos/ooRexx-rust-rewrite` by the same construction, which the
   old literal (`rexx-extract/src/docs/classes.rs` before this fix) contains as a
   prefix -- so the assertion's failure there follows from the same mechanism I ran,
   not from a fresh, unverified inference.
3. **The mechanism the construction depends on.** Read `gate_table_c.rs`'s `run_probe`
   (`gate_table_c.rs:877`-`900`): it computes `abs = fs::canonicalize(corpus.join(probe))`
   -- the real, on-disk committed `.rex` file -- and hands that same `abs` to both
   `run_gate_probe(&abs)` (which calls `run_program(path, ...)` with that exact path
   string) and `oracle.run(&abs)`. `support/oracle.rs`'s `wrapped()` passes that same
   `path` as the oracle's argv and sets `current_dir(path.parent())`. Neither side
   stages, copies, or rewrites the probe: both engines run the actual corpus file in
   place, from its own directory, invoked with its own path -- which is why
   `filespec('path', .context~package~name) || '../fixtures/...'` resolves correctly on
   both sides. This is not a coincidence of the two layouts it happened to be tried in;
   it is what the harness does today, unconditionally, confirmed by reading the only
   two call sites that invoke a probe. **If a future change staged or copied probes
   before running them** -- `method_bodies.rs`'s own `staging_root()` mechanism
   (`method_bodies.rs`, `fn instance_receiver` neighbourhood) is exactly that pattern,
   already live in this same test tree for a different table -- `.context~package~name`
   would resolve inside the staging directory, `..`  would not have a `fixtures/`
   sibling, and the row would break for a reason unrelated to StreamSupplier's own
   correctness. Nothing today asserts that `gate_table_c.rs` never stages, so this is a
   real but currently dormant coupling, not a defect in this fix -- worth a one-line
   note at the `CONSTRUCTION_PROGRAMS` entry or the README saying the expression
   assumes an in-place probe, so the next person who touches `run_probe` sees it.
4. **Anything else machine-dependent.** Checked the rest of `CONSTRUCTION_PROGRAMS`
   (File and Stream's literal paths don't need to exist, verified previously; nothing
   else in the table is a real filesystem path) and grepped every file under `rust/`
   for `/home/moritz` or this checkout's own path, restricted to files these two
   commits could plausibly touch: the only remaining hits are `tests/support/oracle.rs`'s
   `oracle_root()` (pre-existing, documented in `rust/CLAUDE.md` as a deliberate,
   accepted machine property of the oracle binary's location, untouched by either
   commit) and historical `docs/superpowers/plans/*.md` records of past runs (prose,
   not code, also untouched). StreamSupplier was the only in-code, in-test-data
   instance across what these two commits changed.

No other findings.

## Addendum: the adjacent gap (someone else's real path)

Asked to assess a broader rule: an absolute path literal in `CONSTRUCTION_PROGRAMS` is
allowed only where it does not exist on the machine running the test, rejecting a real
path belonging to *any* machine (a contributor's `/home/alice/...`, a container path),
not just this checkout's own.

**Recommend adding it, scoped to `CONSTRUCTION_PROGRAMS` only, alongside (not
replacing) the current checkout-root check.**

- Confirmed it buys real coverage without cost: scanned the live table
  (`rexx-extract/src/docs/classes.rs`) for every quoted literal -- the only absolute
  paths left are `FILE`'s and `STREAM`'s `/nonexistent-gate-table-c-*` (deliberately
  absent, would pass trivially) and `STREAMSUPPLIER`'s own literal is now the relative
  `'../fixtures/streamsupplier_seed.txt'` (not absolute, would not even be examined).
  Zero false positives against the table as committed.
- It would have caught the original defect earlier than the checkout-root check does:
  this table's own convention (stated in `a4b6a37ce`'s commit message and the brief) is
  that every entry is "checked against the oracle from a fresh directory before being
  committed", which means the author already runs the construction locally pre-commit.
  An exists-check fires in that same run, in the author's own environment, where a
  mistaken absolute path is exactly the kind that exists -- turning this into an
  immediate local failure at authoring time rather than a defect that survives until
  someone else's checkout (or a re-review) finds it.
- The two checks catch different shapes and neither subsumes the other: the
  checkout-root check is deterministic regardless of current filesystem state (it would
  still catch a self-referential leak even after the referenced file is later deleted);
  the exists-check is run-environment-dependent but catches paths that were never
  derived from any repo root at all. Keep both.
- **Do not extend this to `method_bodies.rs`'s `RECEIVER_OVERRIDES`.** That table's
  `/`, `/dev/null`, `/etc/hostname` are real, existing, OS-standard paths by design --
  it sends documented methods that need to read or open something, where gate table C
  only sends `hasMethod` and (per its own stated principle) should never need a real
  file at all except through the portable pattern this fix established. The rule is
  right for `CONSTRUCTION_PROGRAMS` specifically because that principle is specific to
  it.
