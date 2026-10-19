# Task 6.8 re-review: fix round 2

Scoped re-review of `d0a1273d3..99bd3ce98`. The code commit is `f48b4d866`.

**Method.**
* **Build.** HEAD was built into the session scratchpad (`rev68r2-target`, `cargo build --release --bin rexx`, exit 0).
* **Probes.** All ran from one fresh `mktemp -d` (`scratchpad/rev68r2-probes.ABnq/`). That directory is not durable, so the lines that decide each point are quoted below.
* **Runs.** The rules are those of the earlier reviews:
  * the oracle wrapper;
  * `memcap 2G timeout 20` on every crate run;
  * stdout, stderr and rc compared separately.
* **Comparison binaries.**
  * "Pre-task" is `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.7-r1/rexx-head`.
  * "Round 1" is `.../cg-9d/t6.8-r1/rexx-head`.

## Round-1 Importants 1 and 2: closed

Round 1's probe `r1.rex` now matches the oracle on stdout, stderr and rc. The cases that were 98.995 in round 1 now bind, and the caller reads `set`:
* `innerThenOuter`;
* `innerTwiceOuter`;
* `deep1` (a reference passed on through three procedures).

`dropThenReuse` still gives 98.995, as on the oracle.

The first review's probes for USE ARG (`u1`, `u2`, `u3`) and the NOVALUE probe (`e4`) also match the oracle. `e3`'s `pr` line (`procedure expose i (a.i)`) is the one difference left, and it is the list ordering the first review queued as Minor 3.

## Hunting "the name is always the owner's"

Only two call sites create a cell, and both go through `promote_slot`:
* the `>name` term (`run/call.rs:1320`);
* the native API's `variable_reference` (`surface.rs:511`).

Neither EXPOSE, USE LOCAL, PROCEDURE EXPOSE nor GUARD creates a cell.
* An exposed or auto-exposed name answers `VarRefHome::Instance` before `promote_slot` is reached.
* PROCEDURE EXPOSE aliases a slot to the caller's slot of the same name.

`RootSet::promote` moves the storage at the end of that alias chain. The name it is given is the referencing frame's spelling, which is the same name. So I found no path that gives a cell any name other than its owner's.

**Probe results (`s1.rex`).** Each case matches the oracle unless noted.

| Case | Result |
|---|---|
| `peThenRef`: a reference taken through a `procedure expose q.` alias of a USE ARG alias, bound twice | `answers ok` |
| `peOfAliasTwice`: a procedure-exposed alias rebound | `answers ok` |
| `procExposedRef`: `>abc.` from a `procedure expose abc.` frame, bound twice | `answers ok` |
| `dropRecreate`: `drop q.`, then `q.7 = 'x'`, `drop q.7`, then a new reference bound twice | `98.995 ... "Z."` on both sides; the caller reads `e.1 Q.1` |
| `interpRef`: `interpret 'call bindTwice >q.'` | `answers ok` |
| `byValueThenRef`: a stem passed by value, then `>x.`, bound twice | `refused 98.995` |
| `refAfterReturnStored`: a reference created in a procedure that has returned, stored in `.local`, then bound twice | `answers ok` |
| Method `expose ov.`, then `>ov.` | `refused` |
| Method `use local lv.`, then `>lv.` | `bound` |
| Method `expose gv.` under `guard on`, then `>gv.` | `refused` |
| `ctxVars`: `.context~variables` | Differs; see Minor 1 |

The case where the owner returns is the `refAfterReturnStored` row.

**Size.** The cell's name does not grow other objects.
* `size_of::<Body>()` is 80 at HEAD, measured with a scratch crate against `rexx-core` and then deleted. That equals the existing `<= 80` bound.
* `Cell(u32, Box<[u8]>)` is 24 bytes, so the enum's size is unchanged.
* Only the cell object holds the name. The cell's storage in `RootSet` and ordinary variables are untouched.
* The name is bytes, so there is nothing new to root. The collector's three `Body::Cell` arms ignore it.

## Strengths

* **Perf.** I ran `callgrind.sh -r 1 -B 0.5 -p "interpret churn sayloop"` against the pin, round 1 and HEAD (exit 0). The table gives each figure against the pin; the last column is HEAD minus round 1.

  | Program | Round 1 vs pin | HEAD vs pin | HEAD minus round 1 |
  |---|---|---|---|
  | interpret | -2.1689 | -2.1653 | +0.0037% |
  | churn | +0.1139 | +0.1140 | +0.0001% |
  | sayloop | +0.0474 | +0.0456 | -0.0019% |

* **Tests.**
  * `cargo test --release -p rexx-exec --lib`: exit 0, 1152 passed.
  * `-p rexx-core`: exit 0 (includes `collect` and `roots`).
  * `REXX_CORPUS_GATE=1 ... --test corpus`: exit 0, `1096 of 1096 matching`.
  * `-p rexx-parse --test sourceline_oracle`: exit 0.

## Issues

### Critical

None.

### Important

None.

### Minor

1. **Queue candidate, pre-existing: `.context~variables` lists an aliased variable under the wrong name.** The test is `say q.`, then `call r >q.`, with `r: procedure; use arg >z.` followed by `.context~variables~hasIndex('Z.')` and `hasIndex('Q.')`.
   * The oracle prints `stem 0 1`. It lists the alias under the owner's name.
   * The crate prints `stem 1 0`.
   * The simple-variable form behaves the same way: `simple 0 1` on the oracle, `simple 1 0` on the crate.
   * HEAD, round 1 and the pre-task binary all agree with each other.

   The oracle's behaviour looks like a defect, because the alias is reachable only as `Z.`. So this needs a ruling, not a copy. It lies outside these clusters.

## Assessment

**Task quality:** Ready.

**Reasoning:**
* Both round-1 findings are closed, judged by the probes that found them.
* Moving the owner's name into the cell removes the lifetime problem: the name now lives exactly as long as the storage it describes.
* No creation path I could reach gives a cell a name other than its owner's.
* It costs nothing measurable, and it does not grow `Body`.

**Verdict: APPROVED** (Minor 1 is a queue candidate).
