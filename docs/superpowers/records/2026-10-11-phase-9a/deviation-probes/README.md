# Probes behind Deviations 31 and 33

`docs/superpowers/plans/phase-4-exclusions.txt` Deviations 31 and 33 cite these. Each program ran
on 2026-10-13 from a fresh directory holding only these `.rex` files, once on the oracle and once on
this crate at the Phase 9a final fix (`rust/target/release/rexx-run`); stdout, stderr and exit
status are kept per side as `NAME.oracle.{out,err,rc}` and `NAME.crate.{out,err,rc}`:

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx NAME.rex )
memcap 2G timeout 20 rust/target/release/rexx-run NAME.rex
```

* `objname-table.rex`: Deviation 31's table, one line per row, except the `::ROUTINE` row: its
  fourth line reads the literal in an internal routine (`r: procedure`). The `::ROUTINE` row is
  `objname4.rex`'s `::routine other` (lines 10 and 25-26), oracle `long`, its eighth output line.
  Oracle rc 0; crate rc 120 at the first `OBJECTNAME=`.
* `objname2.rex`, `objname4.rex`: Task 8b's probes the table was drawn from (`objname2`,
  `objname4` in `.superpowers/sdd/2026-10-11-phase-9a/task-8b-report.md`). Oracle rc 0; crate rc
  120 at the first `OBJECTNAME=` on a string.
* `ord.rex`: Deviation 33's two `instanceMethods` orders, its `enh:` line. Both sides end rc 159 at
  line 8 (97.2, `SETMETHOD` is private from this context) after the two lines the deviation reads;
  the `dir:` line agrees.
