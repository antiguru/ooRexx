# Task 8b callgrind records

`rust/bench-programs/callgrind.sh`, run from `rust/`, against the pin
`/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run` (sha256 `c4b4e8c4…`). Raw outputs are under
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/t8b/`, in the directory each file's last line names.

| file | binaries | what it shows |
|---|---|---|
| `callgrind-full-a.txt`, `callgrind-full-b.txt` | pin, `t8b` = `rexx-run-step4a` (the Step 4 commit, sha256 `a8e6f5af…`) | the full bench set, `-B 0.5 -j 16`, every program within budget, exit 0 |
| `callgrind-step1-dispatch.txt` | pin, base `efbf0fa1d`, Step 1 | dispatch, dispatchclass, sendloop and fibcall after `object_model` was split |
| `callgrind-step2-quick.txt` | base, Step 2 | alloc, churn, sendloop and dispatch once `MethodRows::get` answered a reference |
| `callgrind-step3-heap-accessors.txt` | pin, base, a discarded Step 3 build | the cost of `&Heap` on the variable accessors, which the Step 3 commit does not have |
| `callgrind-mark-loop.txt` | base, Step 3, Step 4 | churn, alloc and heapshape before and after the mark loop's single match |

The full set was split in two to keep each run under ten minutes:

```sh
bench-programs/callgrind.sh -B 0.5 -j 16 -o <out> -p "alloc alloc4c arith assign churn compound decloop decrender dirread dispatch dispatchclass emptyloop extcall fibcall fibfunc" pin=<pin> t8b=<head>
bench-programs/callgrind.sh -B 0.5 -j 16 -o <out> -p "hashnew heapshape intcall interpret nop parse pkglocal sayloop sendloop startup strings textnum varlookup rexxcps pingguard pingmsg pingsem" pin=<pin> t8b=<head>
```
