# Phase 6.1 Task 7 report: refusal dispositions and `closed_phases`

Base `6860d4f80`. Commits: `3b9a80364` (code, tables, records), and the commit carrying this
report and the gate record.

## Disposition table (`rust/corpus/refusal-dispositions.tsv`)

By kind, from `awk -F'\t' '!/^#/ && NF {print $2}' rust/corpus/refusal-dispositions.tsv | sort | uniq -c`:

| kind | constructors |
|---|---|
| GUARD | binary_operator, call_op_off_its_node, chunk_map_too_short, constant_out_of_range, conversion_raised (c7), environment_entry (b18), environment_symbol (b16, b18), expose_receiver, expression, instruction, jump_out_of_range, library_procedure_gone, library_source (b17), loop_op_off_its_node, missing_body, numeric_form_without_expression, object_method, operator_operand (b12), op_not_driven, parse_trigger_operand, register_not_logical, scheduler_inconsistency, select_op_off_its_node, setup_method (b15), signal_op_off_its_node, store_op_off_its_node, unknown_receiver (b9, b13's guard sites), unreadable_collection (b18) |
| DEVIATION | array_index_hole (entry 6), entry_method_without_a_value (new Deviation 29), receiver_class (b13: new oracle-crashes entry 33 and new Deviation 28), stale_handle (exclusions row "A LOCAL HANDLE KEPT FROM AN EARLIER CALL IS STILL REFUSED") |
| LIMIT | chunk_refused (IR plan's `ChunkTooLarge` widths), immovable_reply (Deviation 15), inverted_wait (Deviation 14), unsatisfiable_wait (Deviation 20) |
| REHOME | none. R2's subclass `NEW` refusals are `native_method`, which carries `Phase 9`, so no row is required; the Phase 9 row amendment is Task 12's. |

b11 has no constructor left (its site was `object_position`, deleted by Task 2).

## Constructors

Relabelled to owner `None`: `receiver_class`, `operator_operand`, `library_source`,
`setup_method`. Owner parameter now `Option<&'static str>`: `environment_symbol`,
`environment_entry`, `unreadable_collection` (the `.STREAM` sites pass `None`; `owed[0]` is
`(NIL, None)`, rendered `an entry the library bootstrap fills`; `DirectoryEntry::Owed`,
`owed_entry_owner`, `owed_table_owner`, `unbuilt_collection_owner` carry the `Option`).

Added: `unknown_receiver` (b13's `receiver_kind` `Err` pass-throughs, dead-handle and
"no class of its own" sites, `receiver_class_id` `None` fallbacks, the four "a method object this
crate did not build" argument sites, b9's `context.rs` routine sites, `identities.rs`'s method
context site: 27 sites); `stale_handle` and `conversion_raised` (the shared
`Refused::StaleHandle | Refused::Raised` struct literal split, each with its literal text, which
is the `Display` text it built before); `numeric_form_without_expression` (the `run/settings.rs`
struct literal). `receiver_class` keeps the 41 sites a wrong-type native row reaches: "a value
that is not a(n) array, list, supplier, hash collection, variable reference, class object", "a
package object, routine object, stack frame, executable this crate did not build", "a value that
carries no annotations / no method scope". Site counts from `grep -rn 'receiver_class(\|unknown_receiver('`
over `src/` less tests. Deleted: none. Messages are unchanged except the dropped `(Phase 5)` suffix, and the `owed[0]` placeholder
object renders `an entry the library bootstrap fills` rather than `an entry owed by Phase 5`.

The split is by probe on ours at `6860d4f80` (`/tmp/claude-1000/p61/t7/rc1`-`rc3`): borrowed
rows on an instance reach `not a list`, `not a supplier`, `not an array` (Array, Queue ITEMS,
Array FILL, Queue PUT), `not a hash collection` (Directory, StringTable, Table PUT), `not a class
object` (Class ID, DEFINE), `package object this crate did not build` (Package NAME, LOCAL,
ADDCLASS), `stack frame` (NAME, LINE), `routine object` (CALL), `executable` (Method and Routine
SOURCE), `carries no annotations`, `carries no method scope`, `not a variable reference` (NAME,
VALUE). `rc4`: a `~method` object's `~copy` refuses at `COPY` (Phase 9) before `define`,
`setMethod` or `run` can take it, so the "method object this crate did not build" sites are
guards today.

## The test (`rust/crates/rexx-exec/tests/refusal_dispositions.rs`)

Reads every `fn ... -> Loud` in `src/` (test modules and files excluded), takes the owner from
the constructor's `owned_message` call (`None` or no call: no owner; `Some("Phase N")`; a
`&'static str` parameter or a `COMPUTED_OWNERS` function: given; an `Option` parameter or any
other function: no owner), and requires a row for each ownerless one. It fails on an unlisted
constructor, a row with no constructor, a REHOME whose constructor does not carry its phase, a
non-REHOME row for an owned constructor, an ownerless `Loud` struct literal outside a
constructor, and a cited oracle-crashes entry, Deviation or `docs/`/`rust/` path that does not
exist. `the_check_finds_each_kind_of_disagreement` runs it over a fabricated source and table and
asserts each failure; `the_scanner_reads_each_owner_form` pins the owner forms on the real tree.

Failing first, at `6860d4f80` with no table (`/tmp/claude-1000/p61/t7/step1-fail.txt`): exit 101,
`every_ownerless_refusal_has_a_disposition` failed listing 25 ownerless constructors and the two
ownerless struct literals (`dispatch/library.rs:857`, `run/settings.rs:358`). The first run also
listed `directives.rs:64`, a false positive (closure parameter not recognised), fixed before the
recorded run. Passing at `3b9a80364`: 3 passed. Negative control: the table without its
`receiver_class` row fails with `receiver_class (crates/rexx-exec/src/lib.rs:495): no owner and no
disposition row`.

## `closed_phases`

`CLOSED` is `["Phase 5", "Phase 6", "Phase 7", "Phase 8"]`; the debt paragraph is gone.
`no_owner_table_names_a_closed_phase` scans `tests/{owners,assertions,bif_assertions,keyword_assertions}.rs`
with the literal scan. Before the vocabulary prune and the `Literals` re-home it failed on the
twelve `assertions.rs` rows and `bif_assertions.rs:610` (a report string naming Phase 5 and
Phase 7). `"Phase 5"` and `"Phase 7"` are pruned from `SPLIT_TABLE_PHASES` and both `PHASES`
vocabularies (Phase 7 is closed too and the widened scan names it); `owners.rs`'s zero counts for
`Phase 5` and `Phase 7` tags are deleted, since the vocabulary test rejects such a tag; the
`bif-exempt.txt` header's `Phase 5` category is deleted. Step 4: `git grep -n '"Phase 5"' --
rust/crates` prints only `closed_phases.rs:32`, the `CLOSED` line.

NC-i was already closed by `51e1d37cc` (`Owner:` in any case is read); a fully lower-case
`owner:` row is added to its test. NC-h: a sentence now ends at a full stop followed by
whitespace (so `6.1` and `.rex` do not end it), and a resolution word resolves only where it
opens a sentence, a paragraph or the clause after a colon; the phase is read up to it.
`the_row_check_reads_only_a_resolution_that_opens_a_clause` holds `EXTERNAL was DELIVERED` and
`The other half is FIXED` open, and `: DELIVERED by Phase 6.1 Task 5` and a next-paragraph
`FIXED` resolved. The exclusions file passes under the new rule: `no_open_exclusions_row_names_a_closed_phase` is
green with Phase 5 in `CLOSED`, and the `::REQUIRES` row's "OWNER: Phase 5 for the rest of
::REQUIRES" resolves at its "parse: DELIVERED by Phase 6.1 Task 5". Both queued items carry a closing section.

## `Literals` rows

Each `runDynamicSource` case run outside the harness inside a test-case class with the group's
`q`, `hex`, `bin` and the framework's `runDynamicSource` (`/tmp/claude-1000/p61/t7/lit1/p.rex`):
the oracle prints `1 1 AB AB AB AB AB A A A A A`, rc 0; ours refuses rc 120, `method "NEW" of
class "Routine" is not implemented (Phase 9)`. `lit2`: `.routine~new(n, src)` answers and
`.routine~new(n, src, .context~package)` refuses (`native_executable_new`, `args.len() > 2`). So
the interpreter does not run them, and the twelve rows are re-homed to `Phase 9` with that
reason above `EXEMPT`. In the harness itself (`lit3`) the row has no test case for `self`, and
both engines raise 97.1 on `self~hex`.

## Records

- `rust/corpus/oracle-crashes.txt` entry 33 (b13's crashing rows, from scout A's measured runs
  and Task 4's; nothing re-run to add it).
- `phase-4-exclusions.txt` Deviation 28 (b13's non-crashing rows) and Deviation 29 (value-less
  entry-method assignment, from the Phase 5a upstream-tickets record).
- `rust/corpus/refusal-sites.tsv` re-derived; hand columns written for `unknown_receiver` (agrees,
  no), `conversion_raised` (agrees, no) and `stale_handle` (recorded, yes).
- `bench-programs/alloc4c.rex:6`, `bench-control/alloc4c-101.rex:6`: the quoted refusal is gone.

## Commands and results

All at `3b9a80364`, from `rust/`, each status read unpiped:

- `cargo fmt --all`: no change after formatting; `memcap 8G cargo clippy -j 4 --workspace
  --all-targets -- -D warnings`: exit 0.
- `memcap 8G cargo test -j 4 --workspace --no-fail-fast` (debug): exit 0, 3079 passed, 0 failed
  over 146 test binaries (`/tmp/claude-1000/p61/t7/gates1/workspace.out`).
- `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test
  ir_recorded_oracle`: exit 0; corpus 29 passed, 1 ignored; ir_recorded_oracle 21 passed.
- `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test refusal_sites -- --test-threads=1`
  re-derived the table; `refusal_sites` then 5 passed.
- `refusal_dispositions` 3 passed; `closed_phases` 8 passed; `owners` 6, `keyword_assertions` 7,
  `bif_assertions` 5 passed.
- `git grep -n '"Phase 5"' -- rust/crates`: `rust/crates/rexx-exec/tests/closed_phases.rs:32`
  only.

## Concerns

1. Table ITEMS on an instance is listed in Deviation 28 as the brief asks, but ours does not
   reach `receiver_class` there: `not_this_task` answers `method "ITEMS" of class "T" is not
   implemented (Phase 9)`, a Phase 9 label for a wrong-type row, recorded as such in Deviation 28.
   `Directory~SETENTRY` on an instance answers the same way.
2. The `Literals` rows' `unblocked_by: "Phase 9"` names the interpreter half only; the harness
   runs them outside a test case, so they also need the harness to supply `self` once Phase 9
   lands.
3. Phase 7 was pruned from the vocabularies as well as Phase 5, beyond the brief's wording, because
   the widened scan names it.
4. No performance run: the changed paths are refusal sites and the `owed` owner type
   (`Option<Option<&str>>` on `.environment`/`.local` item reads); this task is not one the spec
   measures.
5. GUARD records for the d-guard constructors cite the census's reason, not probes;
   `call_op_off_its_node` records its one known firing (a concatenated non-first argument), whose
   repro answers `ok` at `6860d4f80`.
