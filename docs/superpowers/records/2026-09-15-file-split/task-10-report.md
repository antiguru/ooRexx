# Task 10 report: the remaining test crates and tools

BASE `8a1c43696`. There are five code commits. Every artifact cited is
under `docs/superpowers/records/2026-09-15-file-split/task-10-files/`
(`files/` below). The tooling is Task 9's, adapted, in `files/tools/`.

## Commits

| # | commit | what moved | from, into |
| --- | --- | --- | --- |
| c1 | `f61c596a5` | the row readers and row types (`read_table`, `Section`, `Construction`, `ClassRow`, `construction_of`, `Edge`, `MethodRow`, the `read_*` functions) | `rexx-exec/tests/gate_table_c.rs` to `tests/table_c/rows.rs` (and `tests/table_c/mod.rs`, new) |
| c2 | `ec22e0ed1` | the probe derivations (the `*_SUBDIR` constants, `*_probe`, `*_probe_text`, the class-probe question builders, `ENTRY_MARKER`, `DOCUMENTED_EDGE_MARKER`, `method_name_literal`, `derived_say_lines`) | `gate_table_c.rs` to `tests/table_c/probes.rs` |
| c3 | `d65b1a843` | the checks (`OracleShape`, `class_probe_shape`, `ENTRY_KINDS`, the `check_*` functions, `Ran` and `run_probe`, `probe_set`, `first_difference`, `argutil_assertion`) | `gate_table_c.rs` to `tests/table_c/checks.rs` |
| c4 | `234125c70` | class-row coverage (`coverage_of`, `unconstructible_index`, `strip_tags`, `collapse`) | `rexx-extract/src/docs/classes.rs` to `src/docs/classes/coverage.rs` |
| c5 | `4118a425c` | method-row derivation (`method_sections`, `Listed`, `listed_members`, `displayed_names`, `names_of`, `Marker`, `TITLE_PARENTHETICALS`, `split_marker`, and the tables only they read: `BARE_NEW_CLASS_SIDE`, `GROUP_HEADINGS`, `TITLE_XREFSTYLE`, `TEMPLATE_MEMBERS`) | `docs/classes.rs` to `src/docs/classes/methods.rs` |

The order follows the brief: the test-only moves first (c1 to c3), then
the leaf with one caller (c4, called only by `class_rows`), then the larger
cluster (c5). Within `gate_table_c.rs` the rows went first because the other
two read them. Each commit message says `git blame -w -C -C -C` recovers the
moved lines.

Line counts, with the plan's `find crates -name '*.rs' | xargs wc -l`
(`files/final/line-counts.txt`):

| file | BASE | final |
| --- | --- | --- |
| `rexx-exec/tests/gate_table_c.rs` | 1890 | 1038 |
| new: `tests/table_c/{mod,rows,probes,checks}.rs` | | 16, 180, 217, 532 |
| `rexx-extract/src/docs/classes.rs` | 1338 | 1000 |
| new: `src/docs/classes/{coverage,methods}.rs` | | 163, 221 |

## What moved, and what stayed, per file

* **`gate_table_c.rs` (c1, c2, c3), as ruled.** The two `#[test]` fns stay,
  so their names do not change. So do `corpus_dir`, `Concept` and
  `CONCEPTS`, and what reads `CONCEPTS` besides the tests (`concept_rows`),
  the phase assignment (`WIRING_PHASE`, `NEVER_AGREES`,
  `UNREACHABLE_STATUS`, `INSTANCE_ARM`, `method_row_owner`,
  `method_row_owners`), the per-row verdict helper `method_row_stdout`, and
  the report (`Measured`, `MethodGroup`, `MethodProgram`, `emit_row`).
  `NEVER_AGREES`'s intra-doc links name `gate_tables::` paths, which
  resolve only where `gate_tables` is in scope; that is one reason the phase
  assignment stays. The three responsibilities the ruling names went to
  one directory module each: `tests/table_c/rows.rs` (the committed row sets
  and their readers), `probes.rs` (where each probe lives and the text each
  holds), `checks.rs` (the structural checks and the probe run).
  `class_probe_shape` and `ENTRY_KINDS` went with the checks, not the
  probes: `ENTRY_KINDS`'s doc links both `class_probe_shape` and
  `check_edge_endpoints`, and a link to an item in another module would need
  an import used only by a doc.
* **The Cargo trap.** `tests/table_c/mod.rs` is a directory module, as
  `tests/gate_tables/mod.rs` is; `gate_table_c.rs` declares `mod table_c;`,
  and `mod.rs` declares `pub(super) mod rows;`, `probes;` and `checks;`.
  Cargo discovers `tests/*.rs` and `tests/*/main.rs`, so none of these
  becomes a target. The target list (`cargo metadata --no-deps`, rexx-exec
  and rexx-extract) was compared with BASE's at every test commit: identical,
  57 targets (`files/base/base-targets.txt`, `files/c<N>/c<N>-targets.txt`).
  Instrument 4 shows the same 60 result blocks throughout.
* **`docs/classes.rs` (c4, c5), as ruled.** "The entry points" read as the
  functions `docs.rs` calls: `Book`, `class_rows`, `method_rows`,
  `unreferenced_sections`, `classes_without_method_rows`, `class_set_rows`,
  `method_set_rows`, `class_set_header`, `class_methods_header`; they stay,
  with `class_sections` (both derivations call it), `render_method` and
  `parse_method`, the public tables (`CONSTRUCTION`, `CONSTRUCTION_PROGRAMS`,
  `UNCONSTRUCTIBLE`, ...), `OPERATOR_PLACEHOLDERS` (read by `render_method`
  and `parse_method` as well as `names_of`), `NO_OWN_METHODS` (read by
  `method_rows`) and the test module. What the two derivations call moved,
  so the `module is src/docs/classes.rs` sentence in both headers still
  names the module whose functions `rexx-extract-docs` runs.
  `rust/corpus/docs/class-set.txt` and `class-methods.txt` are
  byte-identical to BASE's (`git diff 8a1c43696 HEAD -- rust/corpus/docs`
  is empty), and `extract_docs.rs`, which compares them in both directions,
  passed at every commit (instrument 4).
* **Out, as ruled, untouched:** `rexx-bench-suite.rs`, `coverage.rs`,
  `ir_recorded.rs`, `method_bodies.rs`, `native_classes_wiring.rs`,
  `rexx-num/tests/format.rs`.

## Visibility

| item | file | visibility | read by |
| --- | --- | --- | --- |
| `read_sections`, `read_classes`, `read_edges`, `read_method_rows`, `Construction` | `table_c/rows.rs` | private to `pub(crate)` | the tests in `gate_table_c.rs` |
| `Section`, `ClassRow`, `Edge`, `MethodRow`, and every field of each | `table_c/rows.rs` | private to `pub(crate)` | the tests, `concept_rows`, `method_row_owner` |
| `read_table`, `construction_of` | `table_c/rows.rs` | private, unchanged | the readers |
| the `*_SUBDIR` constants, `concept_probe`, `class_probe`, `edge_probe`, `method_probe`, `class_probe_text`, `edge_probe_text`, `method_probe_text`, `derived_say_lines` | `table_c/probes.rs` | private to `pub(crate)` | the tests |
| `ENTRY_MARKER`, `DOCUMENTED_EDGE_MARKER`, `class_probe_entry_questions`, `class_probe_class_questions` | `table_c/probes.rs` | private to `pub(crate)` at c2, to `pub(super)` at c3 | at c2 the checks still in `gate_table_c.rs`; from c3 `table_c/checks.rs` only |
| `method_name_literal` | `table_c/probes.rs` | private, unchanged | `method_probe_text` |
| `OracleShape`, `class_probe_shape`, the `check_*` functions, `run_probe`, `probe_set`, `argutil_assertion`, `Ran` and its fields | `table_c/checks.rs` | private to `pub(crate)` | the tests |
| `ENTRY_KINDS`, `first_difference`, `OracleShape::admits`, `OracleShape::describe` | `table_c/checks.rs` | private, unchanged | the checks |
| `table_c::{rows, probes, checks}` | `table_c/mod.rs` | `pub(super)` | the crate root |
| `coverage_of`, `unconstructible_index` | `docs/classes/coverage.rs` | private to `pub(super)` | `class_rows` |
| `strip_tags`, `collapse` | `docs/classes/coverage.rs` | private, unchanged | `unconstructible_index` |
| `method_sections`, `listed_members`, `displayed_names`, `names_of`, `Listed` and its fields | `docs/classes/methods.rs` | private to `pub(super)` | `method_rows` (and the test module, through the parent's import) |
| `Marker`, `split_marker`, and the four moved tables | `docs/classes/methods.rs` | private, unchanged | `names_of`, `displayed_names` |

`pub(crate)` rather than `pub(super)` in `table_c/`: the crate root is the
reader, two levels up, and `pub(super)` from `table_c/rows.rs` reaches only
`table_c`. The children read the root's private items (`corpus_dir`,
`Concept`, `gate_tables`, `support`) with no widening, since a private item
is visible in every descendant. Nothing is `pub use`d: no existing path had
to keep working (every moved item of `docs/classes.rs` was private; the
public API manifest is unchanged, below). Clippy
`--workspace --all-targets -D warnings` passed at every commit, so no import
is unused and no visibility is too narrow to compile.

## Pinned items

* **Path pins** (`files/path-pins-base.txt`, the brief's two greps per file
  and three wider ones, before any move): no test, source or bench names
  either file as a path it opens. `ir_recorded.rs` reads `tests/*.rs` (not
  recursively) for `ootest/ooRexx/base/`, and `owners.rs` reads
  `tests/coverage.rs` and `tests/loud.rs`; `gate_table_c.rs` holds no such
  marker (`/bin/grep -c` 0), so nothing either scans moved. The prose pins
  are ruled in each commit's `comment-pass.md` and summarised under The
  instruments.
* **`unsafe`**: `unsafe: NONE` at all five commits
  (`files/c<N>/c<N>-unsafe.txt`); `verify10.sh` refused a commit otherwise.
  `unsafe_sites.rs` names neither file.
* **`refusal-sites.tsv`**: each commit re-derived it; every column but
  column 4, and the header lines, identical, 247 rows, column 4 unchanged
  (`files/c<N>/c<N>-refusal-sites.txt`), and `git status` showed no change.
* **The public API** of `rexx-extract` (`docapi10.sh`, Task 9's `docapi.sh`
  for this crate): the manifest is BASE's at every commit
  (`files/c<N>/c<N>-docapi-diff.txt`, empty). See Tooling for the one
  normalisation it needed.
* `environment_seam.rs` and `dispatch_seam.rs` scan `src/` of rexx-exec,
  which this task does not touch.

## Departures from the brief

1. **One corpus prose line edited, at c2.** `corpus/gate-tables/README.md:95`
   said "`gate_table_c.rs`'s `class_probe_text`, `edge_probe_text` and
   `method_probe_text` are the definition ...", and c2 moved all three to
   `tests/table_c/probes.rs`. It is documentation, not a corpus program or
   a record, and the move made it false, so it was corrected in the same
   commit: the one path, no re-wrap. Nothing reads that file
   (`/bin/grep -rn README` over `crates/*/tests` and `crates/*/src`). It was
   declared in `files/c2/extra` and checked by the new `extra_edits10.py`.
2. **A narrowing in an earlier child, at c3.** The four probe items the
   checks read had to be `pub(crate)` at c2, because the checks were still
   in the crate root; at c3 they narrow to `pub(super)`. The alternative
   orders each edit an earlier child too: checks first would have had
   `checks.rs` import them from the root at c2 and rewrite that import at c3.
   The narrowing is declared in `files/c3/extra` (four substitutions, each
   matching once) and checked by `extra_edits10.py`
   (`files/c3/c3-extra-edits.txt`).
3. **Layout fixed by hand at c2.** The mover separates moved units with a
   blank line and takes the blank line around each; for the four adjacent
   `*_SUBDIR` constants that separated them in the child and joined
   `corpus_dir` to the next item in the parent. I rejoined the constants and
   put back the parent's blank line: whitespace between units, which
   instrument 1 (a) lists as an insertion and (b) does not see.
4. **Module docs** on the six new files, one or two lines each, as Task 9's.
   No existing comment was edited except the README line in (1); the
   comment passes found nothing else made false.
5. **Two rustfmt re-wraps**, of widened signatures that crossed the line
   width: `check_edge_endpoints` (c3) and `names_of` (c5). Instrument 1 (b)
   reports each as whitespace-only; instruments 2 and 3 compare their
   tokens and literals, identical.

## The instruments

Per-commit outputs are in `files/c<N>/`, made by `tools/checks10.sh` over the
working tree before each commit (`tools/prod10.sh`: `verify10.sh`, then
`commit_task10.sh`). `commit_task10.sh` refused to commit unless the staged
`rust/` tree hash equalled the one instrument 4 ran on; all five matched
(`files/instrument4/i4-c<N>.meta`). `verify10.sh` also refused a commit
unless fmt and clippy (`--workspace --all-targets -D warnings`) passed; the
three `cargo doc -p rexx-extract` runs had BASE's warning signatures; the
public-API manifest was BASE's; the refusal-sites columns were identical;
the unsafe count was `NONE`; no file outside the parent, the destinations
and the declared extra edits changed, tracked or untracked (`other edits: 0`,
`untracked: 0`); each declared extra edit matched its rule; `/tmp` had 10 GB
free; `comment-pass.md` existed; and, for c1 to c3, the test-target rustdoc
had BASE's warnings and the target list was BASE's, and for c4 and c5 the
test-module links documented cleanly.

| # | 1 (a) unmoved | 1 (b) moved units | 2 tokens | 3 literals | 4 tests | load before / after instrument 4 |
| --- | --- | --- | --- | --- | --- | --- |
| c1 | 1727 equal; 1 declared deletion, 5 inserted | 11 of 11 (9 widened) | 70 of 70 | 70 of 70; 382 literals; 0 control mismatches | identical | 21.74 21.95 18.94 / 42.00 28.42 22.39 |
| c2 | 1529 equal; 6 inserted; `mod.rs` 1 inserted | 17 of 17 (16 widened) | 61 of 61 | 61 of 61; 353; 0 | identical | 20.11 26.18 23.05 / 25.35 31.20 26.87 |
| c3 | 1030 equal; 2 declared deletions, 3 imports narrowed, 5 inserted; `mod.rs` 1 inserted | 17 of 18, 1 whitespace-only (14 widened) | 40 of 40 | 40 of 40; 291; 0 | identical | 11.46 24.91 25.09 / 14.96 20.38 23.14 |
| c4 | 1195 equal; 3 inserted | 4 of 4 (2 widened) | 156 of 156 | 156 of 156; 579; 0 | identical | 13.84 18.83 22.40 / 2.13 5.82 13.25 |
| c5 | 997 equal; 1 import narrowed, 2 inserted | 13 of 14, 1 whitespace-only (5 widened) | 153 of 153 | 153 of 153; 551; 0 | identical | 5.09 6.05 12.69 / 8.61 10.96 12.11 |

Units counted in instrument 1 (b) are item-tool's: a table's rows count one
each (`TEMPLATE_MEMBERS`), and an `impl` block one per member
(`OracleShape`). Every moved unit is byte-identical to its PRE text once a
widened declaration's or field's visibility is removed, except the two
re-wraps in Departure 5. Instrument 4 ran the brief's
`cargo test -j 4 --release -p rexx-extract -p rexx-exec --no-fail-fast` in
a test worktree with its default target directory: 1679 tests in 60 result
blocks at BASE (exit 0, load 13.33 13.57 14.89 before and 19.88 22.07 18.79
after), identical per test and per block at every commit
(`files/instrument4/`). The worktree carried symlinks to the main
checkout's `build/`, `oodocs/` and `ootest/` (Task 9's Departure 5).

**Docs.** `cargo doc --no-deps -p rexx-extract`: one warning (two warning
lines), in `keyword.rs`, at BASE and at every commit, in all three runs
(public, `--document-private-items`, and that with `--cfg test`), none in
`docs/classes.rs` or `docs/classes/`; signatures BASE's
(`files/c<N>/c<N>-doc-*-sigdiff.txt`, all empty). `tests_links.sh` over
`docs/classes.rs` and `docs/classes/`: 0 warnings at BASE and at c4, c5.
For the test crate, which `cargo doc` never documents and
`cargo rustdoc -p rexx-exec --test gate_table_c` cannot (it fails E0432 on
`rexx_exec`, not linking the library: `files/base/rustdoc-test-probe.txt`),
`testdoc10.sh` replays the target's own rustc invocation as rustdoc with
private items and `cfg(test)`: exit 0 and no warnings at BASE and at c1,
c2, c3 (`files/c<N>/c<N>-testdoc.txt`). The intra-doc links in the new
files are enumerated in `files/final/intra-doc-links.txt`, command inside:
two, `checks.rs:42` (`[`class_probe_shape`]`, `[`check_edge_endpoints`]`)
and `coverage.rs:22` (`[`UNCONSTRUCTIBLE`]`), each inside a run that
reported no warning.

**The comment passes**, per commit (`files/c<N>/comment-pass.md`, from
`comments10.py`'s `c<N>-comments.txt` and `positional.py`'s
`c<N>-positional.txt`), rule on every hit. One comment was made false and
corrected (Departure 1). By hand, per commit, the moved names were swept
unqualified across `rust/crates` and `rust/corpus` for c4 and c5
(`files/c4/c4-unqualified.txt`, `files/c5/c5-unqualified.txt`): no comment
outside the two files names one. For c1 to c3 the moved names are private
to one test binary and every listed hit is ruled in the pass. What was not
corrected, because it is not a comment, is under Concerns 1.

**Re-run with the final tooling** on all five commit pairs from `git
archive` trees (`tools/rerun10.sh`, arguments from each commit's
`files/c<N>/args` and `where`, `files/final/rerun-summary.txt`): all PASS,
every output identical to the committed one.

**Cumulative** (`files/final/cumulative.txt`): for each parent, every BASE
unit is present exactly once at `4118a425c`, in the parent or a new file,
identical modulo visibility (`gate_table_c.rs`: 21 stay, 11, 17 and 18 in
`rows`, `probes`, `checks`; `docs/classes.rs`: 137 stay, 4 and 14 in
`coverage`, `methods`; the remaining BASE units are the `use` lines that
changed). **Comments** (`files/final/comments.txt`, each BASE parent against
the final parent plus its new files): no comment line lost; what was added
is the six license headers and the six module docs.

## Tooling changes and controls

* **`splitlib.strip_field_vis`** also drops a field's widened visibility
  when the field carries a doc comment, whose tokens (`# [ doc = ... ]`)
  stand between the `,` and the `pub(crate)`. Without it instrument 2 fails
  c1 on `ClassRow`, whose `owner` field is documented (control T2).
* **`move_items10.py`**, Task 9's mover with `^` (`pub(crate)`) beside `+`
  (`pub(super)`), and a doubled prefix widening every field of a struct.
* **`comments10.py`**: Task 9's with the crate root directory a parameter
  (`SPLIT_ROOT`); Task 9's reads only `src/` and stops on a `tests/`
  destination.
* **`other_edits10.sh`**, `prep10.sh`, `checks10.sh`, `verify10.sh`,
  `prod10.sh`, `commit_task10.sh`, `run_tests10.sh`, `i410.sh`,
  `rerun10.sh`: Task 9's with the crate and root as parameters.
* **`extra_edits10.py`**, new: a declared edit outside a commit's parent
  and destinations, any path under `rust/` (Task 5's `other_edits.py` sees
  `crates/` only), checked as HEAD's text with each declared substitution
  applied, each matching exactly once, against the working text.
  `other_edits10.sh` excludes exactly the declared paths.
* **`testdoc.py`/`testdoc10.sh`**, new: rustdoc over the `gate_table_c`
  test target, described under Docs.
* **`docapi10.sh`**: `docapi.sh` for `rexx-extract`, plus one
  normalisation. Its first form (`base-docapi-rexx-extract-v1.txt`) made
  control A3 (lines shifted only) fail: `help.html` and `settings.html`
  carry `data-current-crate`, which is whichever of the crate's documented
  targets (the library or one of its three binaries) rustdoc wrote last,
  and it flapped between runs of one tree (`rexx_extract_docs`,
  `rexx_extract_assertions`; `files/fix-docapi/`). It now normalises that
  attribute. BASE's manifest was re-derived three times from an archived
  BASE tree, identical each time, and c1's re-checked three times against
  it, the same each time (`files/base/base-docapi-rexx-extract-run*.txt`,
  `files/fix-docapi/c1-docapi-v2-run*.txt`); c1's original check had
  passed under the first form. c2 to c5 were checked with the fixed one.
* `comments_final10.py` (the comment-line accounting above) and
  `perf_builds10.sh` (Performance), new.

**Controls**, on trees fresh from `git archive` except where a control must
plant in the checkout (those restore by path and show the tree clean
afterwards): every earlier task's (Task 2's five, Task 3b's four, Task 4's
eight, Task 5's seven, Task 6's eight, Task 7's ten, Task 8's fifteen, Task
9's sixty-one, the other-edits controls of Tasks 5 and 6, Task 9's
`controls_docapi.sh` and `controls_untracked.sh`), run three times: before
any tooling change (`files/controls-before/`), after the tooling changes
and before the first move (`files/controls-before-first-move/`), and at the
end (`files/final/controls-final.out` and `controls-*-final.txt`). Every
run caught every control. This task's own:

* `controls_task10.py`, 40 of 40 at the end
  (`files/final/controls-task10-final.txt`), and before each commit in
  `--pre` mode (`files/controls-task10-pre-c<N>.txt`):
  * T1: `strip_field_vis` drops a `pub(crate)` after a field's doc
    attribute, or after two attributes, and keeps one after a `]` closing
    an index or after an attribute standing after `;`.
  * T2: on c1 as committed, Task 9's `instruments.py` and `splitlib.py`
    fail instrument 2 on `struct ClassRow` and on nothing else; this task's
    pass.
  * T3: that field widened and its doc line changed is caught by 1 (b) and 3.
  * T4: a comment planted in `gate_table_c.rs` naming `read_sections` is
    listed under (a) and (c) by `comments10.py`; Task 9's `comments7.py`
    lists nothing (it stops on the missing `src/` destination).
  * T5: `move_items10.py` puts `pub(crate)` on `Edge` and each of its field
    lines and `pub(super)` on `read_edges`, and leaves an unprefixed unit
    verbatim.
  * Part B, per commit (each commit as committed passes, 5): a token
    changed in a moved body (c1, c3, c4, c5); a moved literal changed
    (c1, c2, c3, c4, c5); a moved unit deleted (c1, c2, c3, c4, c5); an
    unmoved parent line changed (c1, c2, c3, c4, c5); two moved docs swapped
    (c1); two fields of a moved struct swapped (c1); a declared deletion
    left undeclared (c1); a space inside a continued literal (c2), which
    only instrument 3 sees; two moved constants' values swapped (c2); a line
    of an existing destination changed (c2, `mod.rs`); a member of the moved
    `impl` block changed (c3); an undeclared import dropped beside declared
    deletions (c3); a moved intra-doc link removed (c4); a moved field's doc
    changed (c5); an import put back as another name (c5). Two expectations
    were corrected after a first run, each kept
    (`controls-task10-pre-c2-first.txt`, `-pre-c5-first.txt`): the space in
    a continued literal expected 1 (b) too, which admits whitespace-only
    differences and defers to 2 and 3; and a changed `TEMPLATE_MEMBERS` row
    expected 1 (b) and 3, but a row's key carries its literals, so the unit
    was reported vanished by 2. Both were caught; the expectations were
    wrong.
* `controls_checkout10.sh`, in the checkout on a clean tree, after c1
  (`files/controls-checkout10-after-c1.txt`, then `-v2.txt` after the
  `docapi10.sh` fix) and at the end (in `files/final/controls-final.out`):
  O0 to O4 (the other-edits check passes clean, fails on an untracked `.rs`
  under `tests/table_c/`, on an untracked file beside `tests/`, and on a
  tracked edit to another test file); D1, D2 (a broken intra-doc link
  planted in `rows.rs` makes `testdoc10.sh` report warnings, and unplanted
  it reports none); A1 to A3 (a public doc line and a narrowed public fn of
  `docs/classes.rs` change the manifest, a shifted line does not); E1 to E4
  (`extra_edits10.py` passes an edit exactly as declared, and fails a second
  change beside it, a pattern matching more than once, and a declared rule
  with the file unchanged). All caught at the end; A3 was missed after c1
  under the first `docapi10.sh`, which is how the flapping page was found.

## Performance

Reduced by the brief: the release `rexx-run`'s `.text` sha256 at BASE and
at the final commit. `tools/perf_builds10.sh` built each from a `git
archive` of the whole repository, every file touched, in a target directory
of its own deleted afterwards; each build log shows `Compiling rexx-exec`
once (`files/perf/builds.txt`, `build-base.log`, `build-final.log`).

| revision | `.text` sha256 | bytes | `Compiling rexx-exec` |
| --- | --- | --- | --- |
| BASE `8a1c43696` | `e86350ff9724ed1470f233e726c6000205ded3bb224a4e595e663f12eb3a45e5` | 2522699 | 1 |
| final `4118a425c` | `e86350ff9724ed1470f233e726c6000205ded3bb224a4e595e663f12eb3a45e5` | 2522699 | 1 |

Equal, as expected: `gate_table_c.rs` is a test, and `rexx-extract` is a
dev-dependency of `rexx-exec` (`crates/rexx-exec/Cargo.toml:77`), so
neither file is linked into `rexx-run`. Load 5.78 9.79 11.51 before the
builds and 9.85 9.25 11.03 after.

## Gates

Over `283017495`, this report's first commit, whose `rust/` tree is
c5's. `tools/gates.sh` (Task 9's) ran them after the commit with the
default target directory (`rust/target`), waiting before G4 and G6 until
the one-minute load was under 15 and the five-minute under 40. The tree did
not change from start to finish (`tree-changes 0` before and after,
`files/gates/status.txt`). Each status was read unpiped. The counts come
from `tools/test_results.py` over each gate's own log
(`files/gates/test-*.results`), and the 604 from that log's differential
report (`files/gates/corpus-differential-*.txt`).

| gate | command | result | load average (1, 5, 15 min) |
| --- | --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 | 8.48, 8.92, 10.65 before the run |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings`, empty target dir | exit 0 | |
| G3 | `cargo build --workspace --all-targets --release`, no cap | exit 0 | |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast` | exit 0; 135 result blocks; 2663 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 14.92, 16.81, 13.78 before; 8.64, 14.10, 14.01 after |
| G5 | `cargo build --workspace --all-targets` (debug) | exit 0 | |
| G6 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0; 135 result blocks; 2664 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 8.64, 14.10, 14.01 before; 13.17, 15.17, 14.93 after |

Every figure matches BASE's (135 binaries, 2663 / 0 / 4 release,
2664 / 0 / 4 debug, 604 of 604 STRICT). Nothing was re-run:
`introspection_arity::every_unstable_row_is_really_unstable` passed in
both test gates.

## Concerns

1. **String literals and derived corpus text still name `gate_table_c.rs`
   as where the derivation lives.** None is a comment, and each is behaviour
   or its committed output, so none was changed:
   * the probe-text literals, now in `table_c/probes.rs`, write "Derived
     ... by crates/rexx-exec/tests/gate_table_c.rs, which re-derives this
     file on every run" into every committed probe under
     `corpus/gate-tables/{classes,hierarchy,methods}/`; changing them means
     re-deriving those files;
   * `check_probe_text`'s failure message, now in `table_c/checks.rs`,
     says "The derivation in gate_table_c.rs is the definition of this
     program";
   * `rexx-extract`'s `class_set_header` literal, and so
     `corpus/docs/class-set.txt:29` and `:38`, say `gate_table_c.rs`
     derives the instance-arm probe and reads the `method-owner` column.
   Each stays true read as "the `gate_table_c` test target", which does
   both, and false read as the file. A follow-up that re-words them has to
   re-derive the probes and `class-set.txt` in the same commit.
2. **`gate_table_c.rs` is 1038 lines**, over the trigger: the two tests (the
   table-building one is most of the file), `CONCEPTS` (ruled to stay
   beside what reads it), the phase assignment and the report. The
   ruling keeps the tests in this file; the report code inside the second
   test is the only further cut, and it is one function's body.
3. **`names_of` and `displayed_names` are tested in `classes.rs`'s test
   module** and defined in `classes/methods.rs`: the tests stay because
   moving them would change their names (`docs::classes::tests::*`). They
   reach the two functions through the parent's private import, which a
   glob `use super::*` picks up.
4. `docapi.sh` (Task 9's, for `rexx-parse`) has the same exposure to
   `data-current-crate` only if the documented crate has more than one
   target; `rexx-parse` has one library target and Task 9's manifests were
   stable, so this is noted, not measured.
