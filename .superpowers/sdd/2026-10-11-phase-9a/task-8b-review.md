# Phase 9a Task 8b review

Range `8d3aa11d9..823db3ab0`, 20 commits. Reviewed in passes: (1) `rexx-core`/`rexx-classes`;
(2) `dispatch.rs`, `class_protocol.rs`; (3) `activity.rs`, `lib.rs`, `environment.rs`;
(4) `identities.rs` (Package~copy, `.ROUTINES`, REXX routine executables); (5) `run/call.rs`, `context.rs`,
`executable.rs`, `hash.rs`, `install.rs`, `security.rs`; (6) `object_protocol.rs`, `introspection.rs`,
`value.rs`; (7) `corpus.rs`/support harness; (8) the Deviation, crash-file, disposition and list text.

Probes: a `rexx-run` built at `823db3ab0` (`CARGO_TARGET_DIR=/tmp/claude-1000/p9/t8b-review-target`,
`-j 2`, `Compiling rexx-exec` seen), driven by `/tmp/claude-1000/p9/t8b-rv/both.sh` (oracle under the
standard wrapper from a fresh `mktemp -d`, crate under `memcap 2G timeout`, three descriptors compared
with `cmp`). Probe files in `/tmp/claude-1000/p9/t8b-rv/probes/`, outputs in `/tmp/claude-1000/p9/t8b-rv/out/`.
`oracle-crashes.txt` was read first; entry 8's program was not run.

## Verdicts

**Spec: PASS with one Important.** Every Step 1 item has a witness, and all ten new witnesses are
byte-identical on the three descriptors at HEAD (run here, not taken from the report). The remaining
refusals, the REXX package's own copy and a library routine frame's `~executable`, are ledgered 9d.
The one Important item is a loud refusal of an oracle rc 0 answer that is filed under a crash licence
that does not cover it (I1).

**Quality: changes requested.** The code is sound on GC and the hot paths. Package~copy leaks one
program per copy, and the time grows quadratically (I2). Minors are below.

## Rules R1-R17

| rule | state | evidence |
|---|---|---|
| R1 identity, `enhanced <id>` | witnessed | `enhanced_runs_new` `class:`/`isA:`/`names:` lines and the 97.1 insert |
| R2 dummy in `~subclasses` | witnessed | `subclasses: 2` plus the two entry rows; `dummy left: 1` after 35.1 |
| R3 class INIT on the dummy, before compile | witnessed | `Thing class INIT for Enhanced Subclass` before `Thing INIT`; the 35.1 case prints the class INIT first. Scout `p10` SAME |
| R4 user / metaclass-instance NEW | witnessed | `Other NEW class, self= Enhanced Subclass`; `metaclass_new` `Meta2 instance NEW self= Enhanced Subclass` |
| R5 INIT once, args after the first | witnessed | `enh INIT ... 1 2`, `init:super` reaches Thing's |
| R6 dummy class during NEW/INIT | witnessed | `enh INIT: Enhanced Subclass | an Enhanced Subclass`, `after: THING ... 1` |
| R7 `.nil`-scoped methods | witnessed in part | `method scope: The NIL object`; scout `p9a` (`.nil` item hides) and `p9e` SAME. `p3` differs in order only (multiset-equal, Deviation 33). `p2` differs in count (`instanceMethods(.Object) items: 32` vs `31`, `INIT` repeated): the shadowed-entry 9d item |
| R8 copy | witnessed | `copy: enhanced THING foo: THING 1 1`; array copy in `enhanced_primitive_receivers` |
| R9 SUPER of `.nil` scope | witnessed | enhancing, `setMethod` and plain `setMethod` lines. Class side checked by probe `clssuper.rex` (class `setMethod` and `.class~enhanced` both `super= The Class class`, SAME) |
| R10 primitive receivers | witnessed | `enhanced_primitive_receivers`. Also probe `menh.rex` (Method, Routine, MutableBuffer, Queue receivers), SAME |
| R11 abstract | witnessed | `abstract: enhanced ABS` |
| R12 arguments and errors | 35.1 by code only; the `"&1"` insert is pre-existing (D6) |
| R13 | trivial, not separately witnessed |
| R14 metaclass NEW, double INIT | witnessed | `metaclass_new`: directive (`Counted INIT for K` twice), `~subclass`, `~mixinclass`. Scout `p14` SAME |
| R15 `.class~enhanced` | witnessed | `id: Remote Class ... metaclass: Enhanced Subclass` |
| R16 | Deviation 32 | `mark_enhanced` hands a foreign answer back (`class_protocol.rs`) |
| R17 memory | measured in the report only (Method-object source), no test |

## Findings

### Critical

None.

### Important

**I1. A metaclass `NEW` that answers an existing class is refused, and the refusal is filed under crash
entry 8, which covers only a non-class answer.** `rust/crates/rexx-exec/src/dispatch/class_protocol.rs:774`-`:777`
filters on `is_class && is_cloned`, and the comment above the filter says the oracle crashes (rc 139). That
holds for `'x'`. For a class answer it does not hold. Measured, probes `metaarr.rex` and `metaold.rex`:

* `::METHOD new CLASS` / `return .array`, then `.object~subclass("k", .MyMeta)~id`: oracle rc 0, `id Array`;
  crate rc 120, "a metaclass whose NEW answers something other than a new class".
* `return .Foo` (`::CLASS Foo SUBCLASS Directory`): oracle rc 0, `id FOO Object 1` and on a second
  subclass `id FOO 1` (it rewires `.Foo` to subclass `Object`); crate rc 120.

The pin `a_metaclass_new_answering_no_new_class_is_loud` (`dispatch/tests.rs:1857`) asserts the `.array`
refusal. The `receiver_class` DEVIATION row (`rust/corpus/refusal-dispositions.tsv:52`) and
`oracle-crashes.txt:279`-`:283` justify it with entry 8. Refusing is right (the oracle damages a shared
class, the R16 shape), but no Deviation records it and the cited licence is false for this case.
Fix: either a Deviation for "a metaclass NEW answering a class `Class~new` did not just build is
refused" with these two probes, or narrow the comment and disposition text to the non-class answer and
give the class answer its own row. Controller ruling needed on which.

**I2. Each `Package~copy` keeps a program for good, and a loop of copies is quadratic.**
`Interp::copy_package` (`rust/crates/rexx-exec/src/environment/identities.rs:1031`, push at `:1171`)
appends to `programs`, clones every per-program map, and roots the copy's `.ROUTINES` table, annotation
table and package object with `add_global`, none of which is ever released. Measured, probe
`pcopyloop.rex` (`q = p~copy` in a loop):

| copies | oracle max RSS / wall | crate max RSS / wall |
|---|---|---|
| 20000 | 20316 kB / 0.01 s | 132188 kB / 2.52 s |
| 80000 | 20324 kB / 0.02 s | 466780 kB / 64.63 s |

Memory is linear (about 5.6 kB per copy), time is quadratic (4x the copies, 26x the time). By reading,
`RootSet::add_global` (`rexx-core/src/roots.rs:127`) scans every global by string compare before
pushing, which alone is quadratic in rooted copies; not isolated by measurement. The ruling chose
option (a) for the missed-map risk; this is its lifetime cost, and the ruling says "if wrong, (b)".
Fix or licence: a controller decision. At least the quadratic part should go (a keyed global, or one
global holding the copies).

### Minor

1. **Copy id taken before the tables are copied.** `identities.rs:1032` sets `copy = programs.len()`,
   then `copy_package_tables` sends `COPY` and builds tables before `programs.push` at `:1171`. Nothing
   that runs there can load a program today (plain `StringTable`s, no user code), but the order depends
   on that unstated fact. Push first, or assert `programs.len() == copy.0` before the push.
2. **`Heap::names` is a `Vec` with a linear search** (`rexx-core/src/heap.rs:85`, `object_name`,
   `set_object_name`). Naming N arrays costs O(N^2), and every unnamed array's `~string` and every
   copy scans the whole table once any name exists. The report chose the `Vec` over a map for a
   callgrind reason on the empty case; the `is_empty` early-out already serves that. Low priority.
3. **`Routine~call` allocates the activation's cold block on every call**
   (`dispatch/executable.rs:630`) to record the receiver, including when the receiver is the
   routine's own object. Not on the `CALL` path, so not in the bench set.
4. **Stale doc**: `activation.rs:590` "The `Method` object this method activation runs for" now also
   holds a `Routine` for a routine activation.
5. **`' -- '` in new comments**: `rexx-classes/src/class_graph.rs:276`, `rexx-core/src/body.rs:881`,
   `rexx-exec/src/lib.rs:1307`.
6. **Deviation 33 prose holds mutable state** (`docs/superpowers/plans/phase-4-exclusions.txt:2017`
   "agree byte for byte today", `:2031` "Listed: `lang/package_writes.rex`", which restates the
   `HASH_ORDERED_STDOUT` const and will rot when it changes). Deviation 8's added sentence ("puts the
   multiset comparison back") narrates history.

## Focus answers

**2. Hot paths.** Per send: `Interp::super_scope_in` gains a `scope == NIL` arm on each side
(`dispatch.rs`), and `receiver_behaviour` returns early for `.nil` only (`Primitive::Object` is
`Decoded::Nil` alone, `dispatch.rs:1814`). Per call: `package_routine_lookup` and `routine_object`
branch on `routines_written.is_empty()`; resolution stays per site (`routine_generation` unbumped).
Per `.NAME` miss: `package_locals.is_empty()` guard, then an out-of-line `package_local_entry` that
allocates the index text. Per allocation: nothing (`new_instance` unchanged). Per collection:
`names.is_empty()` then the cold `prune_names`. Per class creation: `builds_in_place` (two lookups)
and `charge_class`. The callgrind table's attributions (dirread through `store_get`, startup through
`install_store`/`builds_in_place`, decloop/parse through `collect_now`) name the same functions; the
dirread `from_utf8` explanation is a layout attribution under 0.15%, which a control would be needed
to call evidence, and it is under every threshold anyway.

**3. GC.** Object-name rows are weak keys: `ObjRef` carries the slot generation, so even an unpruned
row cannot match a reused slot, and only `collect` frees slots (`heap.rs`); `copy_object` copies the
row. The dummy is rooted by `mint_class`'s temp or the `NEW` answer's temp, then by
`ObjectMethods::enhanced_by` (traced in `ObjectMethods::trace`) or `class_owns` for a class answer;
enhancing `Method` objects by temp then `hold_method_object`. The REXX routine executables are
`add_global`ed before anything else allocates. `cold.executable` is traced (`activation.rs:1033`).
Copied packages: rooted for good (I2). The new witnesses are on `phase-9.txt`, which
`collect_stress.rs` reads; not re-run here.

**4. Package~copy.** No `..` in any of the five destructurings (`Interp`, `Activity`, `ObjectModel`,
`EnvironmentModel`, `OptionWrites`). Independent where `PackageClass::deepCopy`
(`classes/PackageClass.cpp:376`-`:411`) copies: routine, public routine, imports, classes, merged
tables, annotations; settings by value as `RexxObject::copy` does. Shared: `.METHODS`, `.RESOURCES`,
the prolog, the security manager, and `~local` when it exists at copy time (a later-built local is
each side's own, witnessed both ways: `0 0 0` then `1 ...`). `package_copy.rex` SAME here.

**5. Deviation 33 mechanism.** Opt-in by `HASH_ORDERED_STDOUT` and `stdout_mode`; the selection
control asserts the list is non-empty and not the whole subset. `stdout_multiset` splits on `\n`, so a
lost final newline, a duplicated line and a changed line all differ, and
`the_stdout_multiset_comparison_discards_ordering_and_nothing_else` (`tests/support/oracle.rs:730`)
asserts each. stderr and rc keep their own comparisons.

**6. Deviations 31-33.** Measured, probes named, scoped to the observable, "OWNER: none", in 28/29's
register. 33's scope paragraph excludes counts and repeats, consistent with the 9d shadowed-entry
item. Prose nits in Minor 6.

**7. Witnesses.** Spot-checked `enhanced_runs_new.rex` and `package_copy.rex` against the oracle and
read both stdouts: each line the report claims is printed and reached. All ten `phase-9.txt` 8b
witnesses SAME at HEAD.

**8. Prose.** No em-dashes, no set sizes found in new comments; ' -- ' and the time words in Minor 5
and 6.
