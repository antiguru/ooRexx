# Phase 9b Task 3 review: the RexxUtil stem routines

Reviewed `2b59b029d..ef1357a27` from the review package. Probes ran from fresh empty directories under
`/home/moritz/dev/repos/t3rev.a3uT/` (deleted afterwards). The oracle ran under the standard wrapper,
the crate under `memcap 4G timeout 20`. The crate binary is `rust/target/release/rexx-run`, built
09:48:51, 8 s before `28aa77b29` was committed. Mutants and collect-every-allocation probes ran in a
`git archive HEAD rust interpreter` copy with its own target dir, so this checkout was not touched.

### Spec Compliance

- ✅ Spec compliant. The five rows are switched to `util_run` (`internal_routines.rs` hunks at diff
  2571 and 2600). The bodies are in `builtin/rexxutil/stems.rs`. The four witnesses are listed in
  `phase-9.txt` and have their `sourceline_oracle` files. Entry 37 is in `oracle-crashes.txt`, and
  the crate-only pin `stem_sort_untrapped_cases/crate` states its reason in its header. The suite
  table shows 44 of 44 tests on both sides.
- ⚠️ Not verifiable from the diff: the lib, table and suite results in the report. I did not re-run
  them. I did re-run the four witnesses (below).

### Strengths

- The stem API is used throughout, with no text munging. Reads go through `stem_object_tail` (no
  default; a tombstone answers `None`), writes through `stem_object_write`, and drops through the new
  `stem_drop_element` (`hash/stem.rs`, diff 2373), which drops only a tail the stem holds. The
  `getContextStem` resolver is shared with the native API (`stem.rs` `context_stem_named`, and
  `library.rs` `context_stem` now calls it).
- Every Review Focus 1 shape is in a witness for all four routines, with the oracle's answer. That
  covers a stem default, a missing element 0, `x`, `''`, `2.5`, `-1`, `2.0`, `' 2 '`, `1E1` and
  `1234567890` as counts, missing tails below the count, `a.x`, `a.1.5` and `a.01`, and a dropped
  tail.
- Re-run at HEAD: all four witnesses give identical stdout, stderr and rc on both sides (rc 168, 216,
  216, 216). I read the stdout and confirmed the focus lines print, for example copy line 29
  (`-1` used as a count), delete_insert line 64 (`1234567890` is a count for Insert) and sort line 64
  (40.922 for Sort).
- `SysDumpVariables` order is compared as a multiset. The witness sorts the file form's lines in Rexx
  (`lines~sort`), and the standard-output form appears only where it prints one line or none. This
  satisfies Deviation 33. The tail snapshot is held (diff 2145-2148), and the test that guards it
  uses a computed value, so it can fail.
- The mutations were honest. A `push_temp` that no test could fail was deleted instead of being kept.

### Mutations run by this review (probe tree, each witness diffed against the oracle's stdout)

- M1: `copy`'s `stem_element_unset(offset)` changed to `(start + offset)`. `rexxutil_stem_copy`
  differs at oracle lines 56-59 (`position "0"`). Caught.
- M2: `element_count` answers a count of 0 for an unset element 0. Copy lines 50-51, delete lines 37
  and 40-41, and insert line 57 change from 40.922 to `ok` or 40.923. Caught.

### Answers to the controller's questions

1. **999999999 count.** The crate's 40.924 comes neither from memory pressure nor from an up-front
   check. `sort` never allocates by count. It walks the elements and stops at the first missing one
   (diff 2058-2061). The oracle's 5.0 comes from `new_array(bounds * 3)` (`StemClass.cpp:1654`), an
   eager allocation that is made before any element is read, so the oracle's answer depends on the
   cap. All runs below were trapped (the untrapped form is entry 37's crash shape):

   | count | oracle, `ulimit -v 1 GiB` | oracle, `ulimit -v 4 GiB` | crate |
   |---|---|---|---|
   | 1000000 | 40.924 | | 40.924 |
   | 30000000 | 5.0 | 40.924 | 40.924 |
   | 50000000 | 5.0 | | 40.924 |
   | 999999999 | 5.0 | | 40.924 |

   The crate gives the answer the oracle gives whenever its allocation fits. **Recommendation:** keep
   40.924 and add no check. Correct the record: this is not the licensed OOM divergence. The oracle's
   5.0 is an artefact of the 1 GiB cap. The licence applies only when a stem really holds that many
   elements, where the crate's `values`/`texts` Vecs would meet real memory pressure.
2. **Stem API and Review Focus 1.** Yes on both counts; see Strengths. Defaults, missing tails and a
   non-numeric `.0` are handled as the oracle handles them, and both mutants above are caught.
3. **Dump order.** It is a hash order (`GetAllContextVariables`/`GetAllStemElements` are Directories),
   and the comparison is a multiset (the witness sorts). Yes.
4. **GC rooting.** The snapshot hold is present and its test can fail. `sort`'s `values` Vec has no
   explicit hold. Two probes under `run_program_collect_every_alloc` found no hazard:
   - A STRING method empties the stem in the middle of the sort (50 collections). The values survived
     because the reqstr latch is armed by any `~new`, so `required_string_dispatch` pushes every
     answer.
   - No latch, 30 heap strings reordered, including the `S.FOO` form (91 collections, correct order).

   No other Vec built in a closure holds unrooted fresh objects: `texts` holds bytes, and `tails` in
   dump is held.
5. **Entry 37.** Accurate. The `StemClass.cpp:1605` citation is correct, and so is the 2026-07-25
   date from memory. The crate-only pin's reason is stated in the case-file header and in the test
   doc. One sentence is wider than its evidence; see Minor 4.
6. **Witnesses fail without the change.** Yes. RED is in the report, and M1 and M2 above were caught.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

1. **A new oracle crash inside RexxUtil is not recorded.** `SysDumpVariables` over a variable whose
   `STRING` method raises gives SIGSEGV (rc 139) on the oracle. This happens even under
   `SIGNAL ON SYNTAX`, after it prints `Name=A, Value='first'`. It ran once and was not repeated,
   because it crashes the oracle. Program:

       signal on syntax
       call p
       exit
       syntax: say 'trapped' condition('O')~code
       exit
       p: procedure
         a = 'first'
         b = .k~new
         call SysDumpVariables
         return
       ::class k
       ::method string
         raise syntax 93.900 array('boom')

   The crate prints `trapped 93.900`, rc 0. Review Focus 3 asks for exactly this kind of shape to be
   recorded. Fix: add `oracle-crashes.txt` entry 38 (with the run count measured by whoever records
   it), add a case to a crate-only pin with its reason, and keep the shape out of every witness.
2. **The report's justification for the 999999999 divergence is false** (`task-3-report.md` lines
   109-111 and Concern 5). The crate's choice is right, but the reason given is not; see question 1.
   The report is a committed record, so correct the sentence. Suggested text: "oracle 5.0 under the
   1 GiB cap from its eager `new_array(bounds * 3)`; 40.924 when the allocation fits (30000000:
   5.0 at 1 GiB, 40.924 at 4 GiB); the crate answers the uncapped value".

#### Minor (Nice to Have)

1. **`sort` writes back by tail; the oracle writes into the elements it captured**
   (`StemClass.cpp:1731-1736`, diff 2081-2083). If a `STRING` method empties the stem during the
   sort, the stem stays empty on the oracle (`[S.1] [S.2]`). The crate re-creates the tails
   (`[K] [heap heap ...]`). This was measured on both, rc 0. It is exotic, and matching it needs
   element identity, which belongs to the object model (9b1). Record it as a known divergence and do
   not patch it here.
2. **`dump_variables` buffers the whole dump and writes it at the end** (diff 2140-2165). The oracle
   writes each line as it goes. A `STRING` method that prints shows `Name=A` before `STRING ran` on
   the oracle and after it on the crate. This is invisible to a multiset of the dump lines, but the
   interleaving differs. It also means a raise partway through discards the lines already formatted,
   although on the oracle that path crashes (Important 1).
3. **`sort`'s converted values are rooted only through the reqstr latch's push.** That holds today:
   user code can run only once some `~new` has armed the latch. If the latch is narrowed, it breaks
   silently. A one-line comment at diff 2062 naming that dependency is enough. The deleted
   `push_temp` was not wrong to delete.
4. **Entry 37's last sentence claims more than was measured.** It says "40.919, 40.920, 40.921 is
   reported untrapped as usual", but only 40.920 is run untrapped (the witness's last clause).
   Narrow the sentence to 40.920, or measure the other two.

#### Out of scope (pre-existing, outside the diff)

- The crate does not send a class-side `::method string class` when it wants a class object's string
  value. It prints `The K class`, while the oracle runs the method (its stem emptied: `[S.1] [S.2]
  [S.3]`). The likely site is the `Primitive::Instance` gate in `dispatch/reqstr.rs`
  `required_string_dispatch`. This is not this task's defect.

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The code is a faithful port through the stem API, and the witnesses cover every
Review Focus 1 shape with the oracle's answer and are shown to fail under mutation. The fixes are
records: an unrecorded oracle crash in `SysDumpVariables`, and a false justification for the
999999999 divergence.
