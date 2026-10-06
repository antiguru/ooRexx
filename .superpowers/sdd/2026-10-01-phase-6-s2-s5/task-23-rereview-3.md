# Task 23 fix round 3, re-review (base cce32934b, head 9c2c6db44)

Probes ran in a `git archive` of `9c2c6db44` (`rust/`, `interpreter/`, `extensions/`, the
`sharing-walks` records) under `/tmp/claude-1000/p6-t23-rr3/`, own target dir, `memcap 8G`,
release, `--features sharing`, through a scratch `#[ignore]` test added to that copy's `mod sharing`
that runs every `.rex` file in a directory through the existing `sharing_of` and prints stdout and
the `SharingReport`. The scratch directory was deleted after.

**Verdict: APPROVED.** C2's remainder and N4 are fixed. The hunt for another walk of the defect
class found none that the records misclassify. A collection in a started activity runs other
activities' readied UNINITs and counts them (N5). The walks README already rules that counted, but
it is the largest effect I found, so the controller may want to confirm the ruling. Two nits about
the enumeration scripts (N6, N7).

## Items

| # | Item | Verdict | Evidence |
|---|---|---|---|
| 1 | C2 remainder (`detach_exposed_tails` rewrite tagged) | FIXED | `stem.rs:573` filter reads through `Heap::peek`, `:607` rewrite through `Heap::peek_mut`; the read of `home` (`:589`) stays tagged. At head, the five `sharing::` witnesses pass (release, `--skip every_ootest --skip derived_list`). Mutations in the scratch copy, one at a time, witness `a_stem_cleared_in_another_activity_shares_no_dead_exposer` only: `:607` back to `get_mut` fails with `program: SharingCount { objects: 212, shared: 103 }`; `:573` back to `get` (rewrite kept `peek_mut`) fails with the same figures. So each of the two reads is witnessed on its own, unlike the collector's two mechanisms (rereview 2's note). `stem.rs` restored by copy, `cmp` clean against the checkout. |
| 2 | N4 debug-checks artefact rows | FIXED | `python3 .../debug-checks.py \| diff - debug-checks.txt` empty at head. The `ir.rs:646` row (the `settings` field) and the `ir/compile.rs:1221` row (a `#[cfg(debug_assertions)] settings,` struct-literal field) are gone; `ir.rs:691` (`setting_at`, `self.settings.get(index).copied()`) lists `copied get`, a slice read. `iterating-functions.py ts.txt \| diff - iterating-functions.txt` empty. |

## New breakage check

- `Heap::peek_mut` (`heap.rs:486`) is `get_mut` without the `#[cfg(feature = "sharing")]
  self.resolved(slot)` line, so feature-off the two are the same code. Release `rexx-run` with no
  features, built from archives of `cce32934b` and `9c2c6db44` moved in turn to one path
  (`/tmp/claude-1000/p6-t23-rr3/th/tree`, `touch`ed, own target dir per tree, each log shows
  `Compiling rexx-exec`), `objcopy -O binary --only-section=.text`, `sha256sum`: both
  `7c00da6895962f58e3a7b416c97efaa8430b6a1a05cc39b787de424b8b475d6f`, 3130350 bytes, the value
  `sharing-off-text-hash.txt` records for `fix3`.
- `git diff cce32934b 9c2c6db44 | grep '^+' | grep -c unsafe`: 0. Em-dashes in added lines: 0.
- `rustfmt --check --edition 2024` on `heap.rs` and `stem.rs`: rc 0. `cargo clippy --release -p
  rexx-core -p rexx-exec --features rexx-exec/sharing -- -D warnings`: rc 0, no warning.
- The new README row, the comment on `detach_exposed_tails`, and the gate sentence agree with the
  code: the filter and the rewrite both read untagged, and a live exposer's rewrite is not counted.

## Hunt for another instance of the class

Every probe has the shape of the existing witnesses. Main makes 100 objects of one kind, keeps them
in `keep`, then runs `say .t~new~start('other')~result` and `say keep~items`. Method `other` never
names `keep` or its contents. Program shared counts:

| main makes 100 of | `other` returns at once | `other` runs the mix below |
|---|---|---|
| small integers (control) | 2 | 6 |
| long strings, `.u` instances, mutexes, event semaphores, stems, `Method`s, `Routine`s, mutable buffers, named `.object`s, `Bag`s, `Directory`s, `Table`s, weak references, open `.stream`s | 2 | 6 |
| `.local` entries with long keys (`'KEEPENTRY'i`) | 2 | 16 |
| session-queue lines, BIF-opened streams, `::class u` methods via `~define`, unsent `Message`s, garbage `.w` (UNINIT) objects, `VALUE`-set variables | 2 | 4 (smaller mix, see below) |
| objects with `setMethod` own methods | 2 | 3 (one send to a fresh `.u`) |

The mix: table, directory, array and relation fills, `SAY`, `LINEOUT` and `STREAM` close,
`PUSH`/`PULL`, `.queue`, `.local~foo`, `.environment~bar`, `.stdout`, `INTERPRET`, a trapped
SYNTAX, a mutex, `s.~empty`, `self~class~methods`, `.context~executable`, a send to a new `.u`,
`SysSleep`, `TRACE R`. The smaller mix is `QUEUED()`, `QUEUE`/`PARSE PULL`, `LINEOUT`, sends to
`.u`, `SysSleep`, 300 string allocations, `SAY` and `VALUE()`.

The one row that grows is `.local`, which I split one operation at a time. With long keys
(`.local['FOO']`, `.local~foo`, `.local~hasIndex`) the count is 7, against 4 with short (inline)
keys. `SAY` gives 4 and `TRACE R` gives 6. The extra objects are the keys on the probed bucket's
chain (`text_slot`'s `to_text`) and `.local`'s store arrays, which main reallocated when it grew
the directory. Both are contents of the directory that the operation names, and the oracle's string
compare reads the same keys. They do not grow with the number of entries.

Started messages (`.u~new~start('v')`, 100 of them) give 202 even with `other` empty. So do alarms
(`.alarm~new(100, ...)~cancel`), each of which starts an activity. These are real: each started
activity touches its receiver and message.

Table growth, as a check of the "hash collection rehash" candidate: main fills a `.table` with 100
long string keys and passes it to `other`. `other` adds 200 more keys and gets 108. A single
`t['a-side key number 7']` gets 8. `expand` (`dispatch/hash.rs:680`) re-hashes every key, so all
100 of main's keys are resolved. They are the contents of the table the operation targets, and
`HashContents::reMerge` hashes every index too. That matches the README's rule for contents, so it
is not a finding.

**N5, Minor (ruling, not blocking): a collection in a started activity runs UNINIT on other
activities' garbage, and those objects count as shared.** Main makes 100 objects of a class with
`::method uninit` and drops them; `other` does `call GC 'Force'`:

| variant | program objects | program shared |
|---|---|---|
| `other` collects | 113 | **101** |
| `other` does nothing (control) | 112 | 2 |
| the 100 kept live in `keep`, `other` collects | 113 | 2 |
| `.w~delete('UNINIT')` (or `~define('UNINIT')`) before the start, `other` collects | 112 | 101 (102 with `define`, which makes one more `.w`) |

`run_ready_uninits` sends UNINIT from whichever activity collected. It goes through
`run_uninit_batch` and then `run_one_uninit`, which calls `answers_uninit` (tagged
`own_method_entry`), `is_class`, and `send_message`. The UNINIT is really sent in each row. With
`say 'U'` in the UNINIT, the oracle prints 100 `U` lines in 3 of 3 runs, with UNINIT deleted or
hidden too (`.w~new~hasMethod('UNINIT')` answers `0`), and this engine does the same. So there is
no read without a send. The walks README rules this counted ("an UNINIT sent to the object
(`run_one_uninit`)"), so the records are not false. But the figure depends on which activity's
allocation triggers the collection, and the activity that collects never names the objects. That
is the shape of the dead-exposer case, except that Rexx code really runs on the object in the
collecting activity. If the controller wants "touched by the program", this is the remaining walk
to pause. If it wants "touched by any activity in this implementation", the ruling stands. Either
way, a sentence saying the count depends on which activity collects would make the record honest
about it.

**N6, Nit: `iterating-functions.py` cannot see a loop that calls a tagged helper.** It lists only
functions that both iterate and call a tagged accessor directly. One call level out, a scan (every
function in the three crates that iterates and calls a function named in `tagged-sites.txt`) lists
99. The interpreter-table walks among them are `run_uninit_batch` and `run_termination_uninits`
(N5, classified through `run_one_uninit`) and the bootstrap builders `build_environment` and
`mint_local_entries`. The rest iterate their own target (hash `expand`/`walk_in`, the
`Body::Instance` branch of `native_entry`, `notify_parties` over the message's parties,
`watch_guard_variables` over the activation's exposures, `end_native_frame` over the frame's lent
buffers). So no classification changes. The README's "the candidates for a table walk" is wider
than the script; narrow the sentence or extend the script one call level.

**N7, Nit: `tagged-sites.py`'s order depends on the filesystem.** `grep -r` walks in readdir order.
In this archive (tmpfs) the output differs from `tagged-sites.txt` in order only (`sort | diff`
empty), so the README's re-derivation `diff` fails on another checkout. Sort the output.

Other walks read and held: `kept_strings.retain` in `drop_loose_kept_strings` (no heap read);
`timers.retain` and the timer root walk (no heap read); `library_programs` loops (class tables, not
objects); `unnotified_messages` and `failed_sends` (membership only, per activity); `check_uninit`
(the receiver); `standard_stream`; `owed_table_owner` (`owed_entry_owner` compares handles, no heap
read); `route_ends_at` (the SAY route's monitor chain, cached per `route_generation`; bootstrap
objects unless the program reroutes `.output`); `class_owns` and `pool_owner` (the receiver class,
or the class whose method object is being made).
