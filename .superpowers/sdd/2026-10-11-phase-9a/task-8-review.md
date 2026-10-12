# Phase 9a Task 8 review

Range `023565ff1..b0628b9be`, controller commits `5b97e7922` and `7a8ee2f54` skipped. Read-only on the
tree. Probes ran from fresh empty directories under `/tmp/claude-1000/p9/t8-review/`, using a copy of the
implementer's `run.sh` with its output directory changed, against a release `rexx-run` built at HEAD.
One A/B used a second build of `023565ff1`, made from a `git archive` into its own target dir. Both
target dirs and the archive have since been deleted. `oracle-crashes.txt` was read first. Entry 21
(`self` in a Table/Directory subclass INIT) is the one near this task, and no probe goes near it.

## Spec verdict: PASS (with one records gap)

Every Step 1 item is implemented and witnessed: String, Stem, Method, Routine and Message `NEW`, with
and without INIT; class; string value; identity; `request('STEM')`; `copy`; Table keys; `sign`; Stem
`empty`; `scope`; `annotations`; `addRoutine`; `~send`. The four queued items are covered too. By
ruling, `Class~enhanced` is Task 8b's. I re-ran all seven witnesses against the oracle at HEAD and
each was byte-identical on stdout, stderr and rc (`string_subclass_new`, `stem_subclass_new`,
`executable_subclass_new`, `message_subclass_new`, `package_subclass_new`,
`class_target_not_a_class` rc 157/157, `hash_subclass_init_capacity`). `phase-9.txt` lists them, and
`corpus.rs` and `collect_stress.rs` both read `phase-9.txt`. Each Stem-native refusal and each
executable/String/Stem/Message `unbuilt_new` call is gone. `unbuilt_new` remains for
`native_package_new` only. Modulo line numbers, `refusal-sites.tsv` differs from base by exactly one
added row, `Raised not_a_valid_class ... off-send-surface`. That is an oracle error (99.949) and not a
`Loud`, so `refusal-dispositions.tsv` correctly gains nothing.

The records gap is the queued files (Minor 1).

## Quality verdict: CHANGES REQUESTED

There are two Important findings: a memory-accounting hole that OOMs where the oracle runs in 28 MB,
and an unmeasured +28% instruction regression on base hash-collection construction. The bench set
cannot see the second one.

## Critical

None.

## Important

**I1. `copy` of a String subclass instance is never charged to the collector: unbounded growth.**
`rexx-core/src/body.rs:1075`: `held_bytes` reports `NativeState::Buffer`'s capacity and 0 for
`NativeState::Text`. `native_string_new` makes up for it with `charge_unheld`
(`dispatch/construct.rs:487`, `lib.rs:2800`). But `copy_object` (`dispatch/object_protocol.rs:661`)
clones the body and charges it through `alloc_charged`, which reads `held_bytes`, so a copied
instance's bytes are counted nowhere. Failure scenario, measured:

    big = .k~new(copies('x', 1000000))
    do i = 1 to 3000; y = big~copy; end
    say 'done' y~length
    ::class k subclass string

Ours: OOM-killed at `memcap 2G`, rc 137. Oracle: `done 1000000`, rc 0, maxrss 28 MB. Two controls
pass on ours: the same loop with `y = .k~new(big)` (56 MB, rc 0), and with a 1 MB `MutableBuffer`
copied 3000 times (56 MB, rc 0). Fix: report `NativeState::Text(bytes) => bytes.len()` in
`held_bytes` and allocate the instance's state through the charged path, so `charge_unheld` and its
"without holding" rule go away. The witness should gain a copy loop that `collect_stress` or a
memory test can see. No existing witness copies a large instance more than once.

**I2. Base `.directory~new` / `.stringtable~new` / `.table~new` got slower; the bench set does not
construct collections.** `native_capacity_init` (`dispatch/construct.rs:81`) now calls
`initialize_store` (`dispatch/hash.rs:1530`). That runs `holds_hash_store` (`:245`), a by-name
`ClassRegistry::lookup` for each `STORE_HOLDERS` entry until one matches, and `lookup` allocates.
After that it runs `read_store`, and only then the `install_store` that `native_hash_new` used to run
directly. A `Directory` matches at the sixth name and a `StringTable` at the seventh. Measured,
interleaved, 3 runs each, wall seconds for `do 1000000; d = .K~new; end`:

| K | base `023565ff1` | HEAD |
|---|---|---|
| Table | 0.99 1.00 1.02 | 1.05 1.08 1.11 |
| Directory | 0.99 0.99 1.00 | 1.27 1.29 1.29 |
| StringTable | 0.96 0.98 0.99 | 1.30 1.33 1.36 |

Callgrind on 50000 `.directory~new`: 591,177,113 Ir before and 755,349,910 after (+27.8%).
`holds_hash_store` accounts for 161,427,191 Ir inclusive, about 3200 Ir per call, of which
`ClassRegistry::lookup` is 72M. The report's "largest increase alloc +0.0589%" is true of the 27
bench programs but not of this path. `INIT` is reached only through the hash classes' own `INIT` rows,
so a cheap identity test against the object model's cached classes (or `is_a` on one cached ancestor
set) does the job. The fix should be re-measured with a construction loop, not only the bench set.

## Minor

1. **Queued files not closed.** `.superpowers/sdd/queued/2026-10-10-subclass-package.md` and
   `-subclass-rexxinfo.md` (tracked), and `2026-09-30-stringtable-subclass-new-arg.md` and
   `-hash-subclass-init-capacity.md` (untracked, ignored dir), are unchanged in the range, although
   the report closes all four with witnesses. Each needs to be removed or to carry a closing line
   naming its witness and commit.
2. **Identity-audit table omits two sites.** `RexxString::upper()` and `lower()` (no-argument,
   `StringClass.cpp:1628`, `:1718`) are `return this` sites missing from the table. They are
   unreachable for an instance, because `stringArgument` hands them a fresh base string. Probed:
   `d~setEntry(.k~new('ABC'), 1); d~allIndexes[1]~class` is `String` on both. They belong in the
   "internal" row. With them the table covers every site `grep 'return this;'` plus the `? this :`
   in `strip` finds.
3. **Extra pre-scan in `changestr`/`caselessChangeStr`** (`dispatch/string.rs`, `string_changestr`).
   `find` runs once before `change` runs its own search, so a base string with a match scans to the
   first match twice. The net bench figure (strings -0.17%) says this is paid for by the no-match
   case. It is noted only so that no later change mistakes it for free.
4. **Shadowed refusal row.** `dispatch.rs:1090` `("Package", "NEW", native_package_new)` sits beside
   `package.rs`'s answering row (the report's concern 4). If it is dead, the `unbuilt_new` site it
   keeps in `refusal-sites.tsv` names a refusal no program can reach. Delete it, or say which row
   wins.
5. **Prose.** Added or rewritten doc lines keep the ` -- ` dash: `construct.rs` (`native_capacity_init`,
   `native_executable_new` and `native_new_file` docs) and `hash.rs` (`initialize_store`'s
   "**observable** --", moved text). The constraint says no em-dashes.
6. **Per-commit check.** The brief asks for the harness groups per commit. The report shows them for
   the final tree only, and the per-task check ran past 5 minutes (stated, load 24).
7. **Pre-existing, not this task:** a Stem receiver (base or subclass) that sends an unknown or
   private message prints an extra `Compiled method "UNKNOWN" with scope "Stem".` traceback line the
   oracle does not (`x = .stem~new('S.'); say x~nosuch`, rc 159 both). It is the Stem sibling of
   `queued/2026-10-10-directory-setmethod-unknown-traceback-line.md`, and the controller may want it
   added there.

## Focus answers

1. **Fast paths.** The plain-stem paths pay nothing new. `stem_of` tests `is_stem` first, and
   `redirect_of`'s new instance arm and `classify_string_conversion`'s instance arm are out of line
   (`instance_redirect`, `instance_conversion`) and reached only by `Body::Instance`. On plain
   strings, `to_text`/`text_len`/`try_text`/`to_number` gain arms that only instances reach. The
   message-path String natives gain a `subclass_text` probe (decode, heap get, match) per call:
   `numeric_receiver`, `string_arithmetic_operator`, `string_binary_operator` via
   `concatenation_or_this`, `op_not`, `d2x`/`d2c` and `MAKESTRING`. Expression operators on base
   strings do not go through these natives, so the oracle's shape holds: the general path is taken
   by instances only. The `return this` changes (`delstr`, `delword`, `strip`, `reverse`, `upper`,
   `translate`, `changestr`) answer the receiver instead of a copy, which is cheaper for base strings
   too. The callgrind figures are plausible for these functions. I2 is the exception, on the hash
   side.
2. **Identity.** Every site was checked against the C++ conditions (`delWord`'s count-0 and
   skip-fails cases, `strip`'s `length == getLength() ? this`, `concatRexx`'s `len2 == 0`, the
   `upperRexx`/`lowerRexx`/`translate` windows) and is witnessed with a building neighbour. Extra
   probes matched the oracle: `lower(1,0)` K, `copies(1)` K, `left(3)` String, `caselessChangeStr('',
   'q')` K, `translate('', '')` String, `x~'||'('')` K, `x~' '('')` String, `.k~new('') || 'q'`
   String, a Table and a Set index keep K. Also probed: `newFile` twice on two subclasses, and a base
   `newFile` after one (`KR1 KR2 Routine 0 0`). This is the in-place `native_as_instance` conversion's
   one risk, a cached object being reclassified, and it does not happen.
3. **Rooting.** `new_instance` and `native_instance` root their result. `base_string` roots the string
   it builds. Stem `remove()` roots both values. `duplicate_stem_store` and `duplicate_executable` root
   each fresh object. `native_as_instance` allocates nothing. `collect_stress` reads `phase-9.txt`, so
   all seven witnesses run under collect-on-every-allocation, and the report's mutation (deleting
   `base_string`'s `push_temp`) is red there. I found no unrooted local. I1 is accounting, not
   rooting.
4. **Out-of-brief changes.** `Stem~remove()` matches `StemClass::remove` (`StemClass.cpp:346`): value
   set to the name, `dropped` untouched, so tails read the name with no NOVALUE. It is witnessed on a
   base stem. INIT sizing matches `HashCollection::initRexx`/`initialize` (`HashCollection.cpp:63`,
   `:78`). Uncaught-error tracebacks for `.table~new(-1)`, `.directory~new(1, 2)`, `.set~new("x")`,
   `.stringtable~new("x")`, `.identitytable~new(1,2)`, `.bag~new(-1)`, `.relation~new(-1)`,
   `.properties~new(-1)` and Table/Directory subclasses with `-1` are all byte-identical to the
   oracle. A forwarding `Directory`/`StringTable` subclass with `new(100)` iterates in the oracle's
   order. Every corpus program that names a hash `~new(...)` with arguments, a hash subclass, a
   no-tail `remove()`, or the changed String methods is in a phase subset, so the gated corpus run
   covered it. The cost is I2.
5. **Queued items.** All four are witnessed and the witnesses match the oracle. The files are not
   updated (Minor 1).
6. **Refusals and tables.** Consistent, as above. The `dispatch/tests.rs` pin now asserts `K K.`.
7. **Prose.** No em-dash characters and no set sizes in code comments. The ` -- ` dashes are Minor 5.
   No comment describes a mechanism that does not exist: the lazy default store
   `native_hash_new`'s doc names is `store_of`'s.
