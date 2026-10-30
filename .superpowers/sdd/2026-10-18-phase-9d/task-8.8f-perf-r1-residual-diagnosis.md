# Task 8.8f performance repair: Residual parse diagnosis

## Scope and measured attribution

The first completed A/P parse pair localizes the regression to `Interp::exec_parse`, whose self cost increases by 5,400,000 instructions. A is `/tmp/codex-p9d-t88-pins/8.8a-perf-r1-rexx`, SHA256 `ecb06e984ed736233ce0e2ef779b74941488ae64fdec10333dfb2215c309198a`, and P is `/tmp/codex-p9d-t88-pins/8.8f-perf-r1-rexx`, SHA256 `6ed1c08eb03d54c48d1fe3947ce376f09502aaeba34ac221c334b01b1572b86e`. Both immutable pins were rehashed, and the current trigger file was verified byte-identical to P source commit `4c11ee4449a1fe7b537c2a216ee7c10b1a5b12ab`. This diagnosis performs no source, index, HEAD, build, interpreter, or profile change and does not inspect the temporary scheduler mutation.

| Round 1 quantity | A | P | Difference |
| --- | ---: | ---: | ---: |
| Own Ir | 1,548,713,610 | 1,554,105,485 | +5,391,875 (+0.34815184%) |
| `exec_parse` self Ir | 628,400,000 | 633,800,000 | +5,400,000 |
| All other own Ir combined | 920,313,610 | 920,305,485 | -8,125 |
| `exec_parse` symbol bytes | 16,479 | 16,486 | +7 |
| Stack reservation, excluding register pushes | 584 (`0x248`) | 664 (`0x298`) | +80 |

The round-1 descriptors report valid status 0, identical stdout SHA256 `4b90e59c1e303baddd84ddfd335185d91757d998e96efb18f9920ad30303f656`, and empty stderr. The inspected checkpoint also contains valid A round 2 at 1,548,713,619 own Ir and P round 3 at 1,554,105,485, but it does not yet contain every descriptor needed for a full three-round comparison. Function self totals sum all inlined source-file attributions while excluding call-edge costs, and an independent sum of `callgrind_annotate --inclusive=no` agrees with the raw aggregation. Consequently the table attributes the observed pair without claiming final medians, SPREAD classification, or performance acceptance.

## Generated inline-text work

The hot pattern is still the inline text `b`, searched twice in each of 200,000 iterations of `bench-programs/parse.rex`. P's combined String/Mixed arm decodes that handle into a separate `InlineText` local, then borrows its bytes before selecting ordinary or caseless search. At P addresses `0x310bed` through `0x310c0f`, LLVM emits twelve moves/shifts to extract bytes, followed by eight byte stores at `0x310c13` through `0x310c47` and another load/store copy at `0x310c4f` through `0x310c57`. Its profile attributes 4,800,000 self instructions to `uint_macros.rs:4058` on this path, exactly twelve instructions per pattern, while A has no `exec_parse` cost at that line.

A's existing `Rendered` path constructs the inline representation more compactly at `0x310916` through `0x310936`. It extracts the length, masks the packed bytes, combines them with an OR, and stores the resulting eight bytes together, alongside Rendered's value and discriminator. P also tests String versus Mixed again at `0x310e88` through `0x310e91`, after the initial combined-arm selection at `0x3109c1` through `0x3109ca`. The frame reservation grows at the function prologue from A `0x30f42a: sub $0x248,%rsp` to P `0x30f8ca: sub $0x298,%rsp`, despite only seven additional symbol bytes.

These instructions show why fewer source-level conversion calls did not imply fewer executed instructions. Source-file self deltas include +5,200,000 in `uint_macros.rs`, +4,000,000 in `parse_template.rs`, +2,200,000 in `trigger.rs`, and reductions of 3,600,000 in `value.rs` and 1,600,000 each in Vec and pointer code. Those redistributed locations sum with the remaining rows to the measured +5,400,000 function total, but they do not isolate independent causal savings for each Rust statement. The scalarized inline representation is a concrete avoidable mechanism, while attributing the entire difference solely to decoder branching, frame size, or the earlier pure move would exceed this evidence.

## Bounded next candidates

The first candidate is to restore A's separate String and Mixed rendering arms and change only the duplicate converted-result retention. Keep `let needle = self.required_string_value(value)?`, condition its following push on `needle != value`, then use the existing `render` and `needle.text(self)` flow for the original search operation. The comparison is derived ObjRef identity, so it cannot send user equality, and the original evaluated value is already pushed before TRACE and conversion in `trigger.rs:36-37`. This retains a converted value whenever it differs, avoids a new decoded aggregate or separate inline-byte lifetime, and leaves the global root API and F1/F2 roots untouched.

The identity guard also covers an unchanged heap string or number, using the existing caller root rather than keeping a duplicate reference. `ActivityRoots::push_frame` records the current temporary length and `pop_frame` discards only values pushed since that mark (`rexx-core/src/roots.rs:455-465`), so nested work must preserve that earlier caller root. `required_string_or` already returns immediate text and SmallInt unchanged and preserves its existing heap/rare conversion roots (`dispatch/reqstr.rs:99-135`), while the candidate still performs the required conversion before comparing its answer. Any conversion error therefore propagates before the guard, and newly returned heap strings remain rooted before rendering or borrowing source bytes.

A narrower fallback keeps the same old rendering arms but skips the duplicate push only when the converted answer is proven immediate. That avoids relying on the unchanged heap-value root argument, but it introduces another decoding/tag condition and may leave more overhead in the hot function. Neither candidate should retain P's separate `InlineText` aggregate or merged-arm search dispatch merely to avoid restoring the prior local structure. Testing one candidate at a time with the existing immediate/heap/default/raising and allocation-stress controls, followed by matched A/candidate measurements, is necessary to choose between them.

Both candidates preserve operand evaluation and its original root, per-line TRACE naming, required-string sends and failures, positional triggers, and the conversion-before-source-borrow boundary. The identity candidate adds no overloaded equality and no assumption that every string-like object is an immediate. No candidate is implemented here, and this static analysis cannot establish emitted code, recovered instructions, or budget passage for an unbuilt variant. The USE and StringValue-probe repairs are outside this residual candidate's scope and should remain unchanged unless separately evidenced.

## Reproduction artifacts

Read-only outputs are under `/tmp/t88f-perf-residual/`, including `a.parse.asm`, `p.parse.asm`, `a.annotate`, `p.annotate`, `symbols.json`, `self-diff.json`, `file-diff.json`, and both `*.parse.lines.json` files. Disassembly uses `nm -S -C PIN` to select `<rexx_exec::Interp>::exec_parse`, then `objdump -d -l -C --start-address=START --stop-address=END PIN`; A starts at `0x30f420`, and P starts at `0x30f8c0`. Self-cost annotation uses `callgrind_annotate --auto=no --threshold=100 --inclusive=no PROFILE`, without collecting a new profile. The raw inputs below identify the exact existing round-1 records, leaving complete matched acceptance to the controller.

| Profile under `/tmp/codex-p9d-t88-performance/h-a-fperf/` | SHA256 |
| --- | --- |
| `parse.a.r1.cg` | `0bc7f38b46b075837d1da732538165a7139235d7b5ee6f8acc355ea8e5a7adc7` |
| `parse.p.r1.cg` | `c13f3397153fddfac58bcfc4516dd3512ddd88c6dbf89a5c745b7801eaf09953` |
