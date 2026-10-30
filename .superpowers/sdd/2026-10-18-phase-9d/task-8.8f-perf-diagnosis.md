# Task 8.8f performance diagnosis

## Evidence and scope

Existing profiles localize the parse regression to `exec_parse` and the directory-read regression to the general hash lookup's new reentry bookkeeping. This diagnosis reads current source at `57fdbedd3bb1de313d30e42a019433f727fd5c02`, the immutable e2/f-r1 executables, and completed raw profiles. It performs no source/index/HEAD change, build, interpreter run, or new profile collection. The candidates below require implementation and own-BASE measurement before any acceptance claim.

The e2 pin is `/tmp/codex-p9d-t88-pins/8.8e2-rexx`, SHA256 `00e29fde4456ee4845a76941263f495b827c083af35363cb54eb3ac8c6b7c733`. The f-r1 pin is `/tmp/codex-p9d-t88-pins/8.8f-r1-rexx`, SHA256 `b695b78fe24903b4be5a2874c8cd128193bbe165f92b50a4d6a8a9956d03d3be`, from runtime source `3e1aa9c879a90214bb5c680357975f58a0081abc`. `/tmp/codex-p9d-t88-performance/e2-f/binaries.txt` binds profile label `f` to that corrected pin. Completed `.run.json` descriptors for both programs and all three rounds report valid status 0 with matching output hashes and empty stderr, although the complete all-program batch remains the performance worker's responsibility.

| Program | e2 own Ir, each round | f-r1 own Ir, each round | Difference | Own change |
| --- | ---: | ---: | ---: | ---: |
| parse | 1,545,246,232 | 1,548,654,364 | +3,408,132 | +0.22055592% |
| dirread | 2,634,713,077 | 2,644,320,661 | +9,607,584 | +0.36465390% |

Dirread was provisional at dispatch, but its third completed pair was present when inspected and matches the first two own counts exactly. The following attribution uses round 1 self costs, summed across every source-file attribution for each function, including inlined Rust library code. This avoids treating moved source lines as newly executed functions and excludes callee edge costs from the sum. The same aggregate is obtained from `callgrind_annotate --auto=no --threshold=100 --inclusive=no PROFILE` and direct aggregation of raw self-cost records.

| Program and function | e2 self Ir | f-r1 self Ir | Difference |
| --- | ---: | ---: | ---: |
| parse, `Interp::exec_parse` | 625,000,000 | 628,400,000 | +3,400,000 |
| dirread, `hash::probe_in` | 38,025,852 | 47,232,951 | +9,207,099 |
| dirread, `hash::merged_get` | 24,803,100 | 25,203,150 | +400,050 |
| dirread, `hash::hash_of` | 25,633,753 | 25,633,753 | 0 |
| dirread, `hash::same_index` | 42,422,554 | 42,422,554 | 0 |

## Parse candidate: Immediate pattern values

`rust/bench-programs/parse.rex:4-11` binds `p0` to the inline string `b` and evaluates two variable-pattern searches in each of 200,000 iterations. It also executes one positional trigger and an ordinary word-splitting template per iteration. Consequently the new required-string boundary runs 400,000 times on an immediate string that cannot send, allocate in the interpreter heap, or raise conversion failure. The profile has no executed `required_string_dispatch` or `assign_expr_target` cost and retains 200,000 calls to `whole_nonneg`, so rare-object conversion and the general TRACE assignment path are not the source of this measured hot-loop increase.

`parse_template/trigger.rs:36` already roots the evaluated operand before tracing and conversion. Lines 63-65 and 70-72 then call `required_string_value`, unconditionally push its answer again, and render it for string or caseless search. `dispatch/reqstr.rs:99-112` explicitly returns an inline string or small integer unchanged before heap lookup or user code, while `value.rs:925-950` decodes that same immediate again into `Rendered`. The ordinary pattern path therefore pays an avoidable duplicate temporary push and repeated immediate decoding, although the fallible conversion remains required for other values.

The narrow candidate is to recognize `Decoded::Text` in the String/Mixed search boundary and search directly with the local decoded inline bytes. For `Decoded::SmallInt`, retain `Rendered`'s local integer scratch representation but skip the identity required-string conversion and duplicate root. Other decodings must continue through the existing required-string helper, with the converted result rooted and consumed before any later conversion, preserving reference/stem/default behavior and raised Failure. Keep operand evaluation, the original root, and trace naming in their current order, and do not modify positional triggers or the source-buffer borrowing contract.

The emitted code confirms that this work survives optimization. In the f-r1 `exec_parse` symbol, the extra temporary push occupies `0x31002b` through `0x310056`, followed by the next ObjRef decode and jump-table dispatch at `0x31005d` through `0x310071`. The symbol grows from 16,035 bytes at e2 address `0x30ddc0` to 16,479 bytes at f-r1 address `0x30ead0`. This proves an ordinary-path mechanism and localizes 3,400,000 of the 3,408,132 added own instructions, but does not prove exactly how much a proposed fast arm will recover after register allocation and branch layout change.

Pure moves cannot be assigned an independent cost from these two endpoint binaries. The old `run.rs` attribution inside `exec_parse` falls from 13,200,000 to 3,200,000 instructions while the extracted assignment file gains 10,000,000, which is a source-location redistribution with zero net change for those rows. `exec_parse` remains one emitted function with the trigger logic inlined on both sides, and per-file counters also shift among roots, iterators, handles, and slice code. A claim that the file extraction itself caused the regression, or that one debug-line cost isolates the extra root exactly, would exceed this evidence.

## Directory candidate: Lookup without callbacks

`rust/bench-programs/dirread.rex:1-10` reads a Directory by message and `[]`, plus a `.local` environment symbol, for 200,000 iterations. Its 400,258 `probe_in` calls and 400,219 candidate-comparison executions include the benchmark's 400,000 general Directory lookups plus initialization. The added work is overwhelmingly inside `probe_in`, while `hash_of` and `same_index` retain exactly the same aggregate self costs. The separately measured `merged_get` increase is 400,050 instructions and must remain visible rather than being silently attributed to conversion.

At `dispatch/hash.rs:683-686`, the new path snapshots `store_generation` around every HASHCODE calculation and can reload the Store. At lines 703-711, it repeats that bookkeeping around every index comparison, carrying receiver, half, saved Store, and restart state through the loop. These checks are required where equality-key HASHCODE or `==` can call user code, but `Keys::StringValue` hashes unsent text and compares unsent text at lines 596-600 and 635-639. The value rendering implementations at `value.rs:713-846`, `:699-706`, and `:1629-1638` use unsent rendering rather than the required-string message protocol, so this string-keyed branch has no reentrant send to reconcile.

The narrow candidate is to dispatch `Keys::StringValue` to a probe loop without callback generation snapshots or restart state, keeping the existing callback-capable path for equality-key collections. Factor the shared slot/bucket bookkeeping without changing the Probe predecessor contract needed by insertion/removal, or isolate the cold restart machinery so it does not inflate the ordinary string-keyed frame. Do not classify every equality-key lookup as safe merely because the queried index is an inline value, because comparison protocols can involve the held value. Leave F1 current-slot removal, F2 validated snapshots, required equality sends, raised callbacks, and their collection-stress controls intact.

The machine code makes this added bookkeeping concrete. `probe_in` grows from 656 to 1,120 bytes, and its stack reservation grows from `sub $0x48,%rsp` at e2 `0x1d43aa` to `sub $0xd8,%rsp` at f-r1 `0x1d435a`. F-r1 loads generation at `0x1d4382`, compares it at `0x1d43e1`, then loads it for candidate equality at `0x1d454d` and compares it at `0x1d4585`, with reload branches following each check. Those source additions explain a directly observed mechanism, while the full 9,207,099 instruction increase also includes additional register/stack/result-handling work and cannot be assigned solely to the four generation accesses.

`hash_of` currently copies string-key text through `to_text(index).into_owned()`, and `same_index` owns both rendered strings before comparing them. That borrowing/copying cost predates f and has zero measured self-cost delta here, so it is not the first candidate for this own-delta repair. Likewise, the conservative f-r1 saved-store roots are in Relation selection and REMOVEITEM paths that this workload does not exercise in its hot loop. Removing those roots would neither explain this profile nor constitute a targeted fix for the observed directory regression.

## Reproduction and limits

Read-only annotation and disassembly outputs are under `/tmp/t88f-perf-diagnosis/`, including `parse.{e2,f}.annotate`, `dirread.{e2,f}.annotate`, `*.self-diff.json`, `*.lines.json`, `symbols.json`, and `{e2,f}.{parse,probe}.asm`. Symbol ranges came from `nm -S -C PIN`, and each listing came from `objdump -d -l -C --start-address=START --stop-address=END PIN`. The round-1 raw inputs and their SHA256 values are listed below to identify the exact immutable evidence. These measurements support scoped candidates, not a prediction that either candidate passes the own-budget gate or that unrelated provisional programs are explained.

| Raw profile under `/tmp/codex-p9d-t88-performance/e2-f/` | SHA256 |
| --- | --- |
| `parse.e2.r1.cg` | `17a9deb8ecbec0b458d0033a327509e3ca143817e48e223a7ebb1a67a3ce3b3b` |
| `parse.f.r1.cg` | `6d876029a01a4e1e3b2de6b6019a14cb72b0ede6574749ce9d528bba25328b65` |
| `dirread.e2.r1.cg` | `2d49de1f463d568bf9da54f2b308dd936a8f09df153993b10b8af0c302dbc7a2` |
| `dirread.f.r1.cg` | `43768b3a31fbc2308bafc994ac069d2323d4ca7fae463849f8d8215ba0aec343` |

## Pingmsg addendum

Completed pingmsg profiles identify a separate `USE ARG` cost that the proposed string-keyed probe fast path will not materially address. The three e2 own counts are 1,648,588,476, 1,648,602,596, and 1,648,598,889, while f-r1 records 1,649,801,038, 1,649,810,564, and 1,649,799,532. Their medians are 1,648,598,889 and 1,649,801,038, a difference of 1,202,149 own instructions or +0.07291944%, refining the provisional single-pair signal. Every inspected descriptor is valid with identical stdout hashes, empty stderr, and status 0, but the complete batch's gate classification remains the performance worker's responsibility.

| Function self costs, summed across inlined source files | e2 | f-r1 | Difference |
| --- | ---: | ---: | ---: |
| `Interp::exec_use`, each of all three rounds | 9,280,917 | 10,481,022 | +1,200,105 |
| `hash::probe_in`, round 1 | 25,716 | 32,751 | +7,035 |
| `hash::merged_get`, round 1 | 3,100 | 3,150 | +50 |

`rust/bench-programs/pingpong/pingmsg.rex:3-14` starts 80,000 messages passing the integer 1, and the target method executes `use arg i`. `run.rs:2087-2098` now independently renders the result and assignment trace lines, adding an unconditional temporary root between them even when TRACE is off. The f-r1 disassembly emits that Vec push at `0x32365a` through `0x32368a`, then `assign_by_name`, then the separate intermediate-trace gate at `0x3236a1`; the additional `Result`/optional-rendering machinery is also visible in the profile's inlined costs. The fixed +1,200,105 `exec_use` self instructions across all three rounds localize nearly all of the own increase despite small scheduling and library differences elsewhere.

The scoped candidate is to avoid retaining immediate integer/text arguments on the temporary-root stack in this USE binding path, while preserving roots for heap/default/converted values and preserving each trace rendering at its existing emission point. The separate trace gates and error propagation must not be collapsed back into shared rendering, and any broader removal of heap roots requires an independent lifetime proof. The StringValue probe candidate can at most address the small initialization-sized probe component evidenced here, not the dominant `exec_use` increase. This is an attribution and candidate only, with no new profile, implementation, or acceptance prediction, and no other hot function is named without a supporting completed profile.

Read-only outputs are `/tmp/t88f-perf-diagnosis/pingmsg.{e2,f}.annotate`, `pingmsg.self-diff.json`, and `{e2,f}.use.asm`, produced with the same annotation/disassembly commands described above. The e2 `exec_use` symbol starts at `0x322130` with 13,125 bytes, while f-r1 starts at `0x3230f0` with 12,973 bytes, demonstrating that smaller emitted code can still execute more instructions on this path. Raw `/tmp/codex-p9d-t88-performance/e2-f/pingmsg.e2.r1.cg` has SHA256 `0b3b8eea93260564e7cba0635eeb39e2fdcf4139fb0c8721e25f172e96d8ba48`, and `pingmsg.f.r1.cg` has SHA256 `da8fff8b2551bf8fb17da90b965f28701c97751cea50a624516ad22d2887c1d5`. No source, index, build, or executable was changed for this addendum.
