# Task 8.8e2 repair round 1: Residual diagnosis

The completed ordinary-send medians leave approximately five own instructions per repeated send above original e1. The [controller's residual table](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/performance/f-registry/residual-vs-original.tsv) compares H against matched F and separately against the earlier e1 cohort. H versus e1 includes intervening e2 and f source changes, so it does not isolate registry cost or establish the result for the remaining programs. This investigation reads H assembly and source without editing Rust, building, running an interpreter, or collecting new profiles. The H pin hashes to `470dcb41db61ab7a4d64f3276ce28e539d5686d130be245a4ec7b30e6d6725ac`, from source `1233b81da730142057fcf810e324d10d0e55c109`.

| Program | H versus matched F | H versus e1 | H minus e1 own instructions | Repeated sends | Difference per repeated send |
| --- | ---: | ---: | ---: | ---: | ---: |
| dispatch | -0.55225710% | +0.12091429% | 25,009,613 | 5,000,000 | 5.00192260 |
| dispatchclass | -0.59654662% | +0.13069467% | 20,009,598 | 4,000,000 | 5.00239950 |
| sendloop | -0.82331095% | +0.18086132% | 25,009,615 | 5,000,000 | 5.00192300 |

## Registry miss and package lookup

The ordinary H registry miss executes four compare/branch instructions in `Interp::begin_invoke`, where `begin_method` is inlined. At `0x26d4f3` it compares the MethodId against the dense length, and the taken `jbe` at `0x26d4fa` reaches the minted-map emptiness comparison at `0x26d521`. The taken `je` at `0x26d529` skips hashing and the hit-only `cold_mut` call, reaching `0x26d616`. These checks distinguish a possibly occupied dense slot from a possibly occupied runtime map, and removing either condition without equivalent information would lose registered executable identities. The four explicit instructions are a static path count, while the approximately five-instruction whole-program residual also includes surrounding code generation and fixed startup differences.

```asm
26d4f3: cmp    %rbp,0xfd8(%r14)
26d4fa: jbe    26d521
26d521: cmpq   $0x0,0xff8(%r14)
26d529: je     26d616
```

The suggested eager `running_activation` cost is absent from the ordinary no-options H path. Source `dispatch/method_entry.rs:105-108` does form the caller-settings argument eagerly, but the inlined assembly first checks `options_armed` at `0x26d26e` and branches at `0x26d276` directly to callee construction when it is false. The running-activation pointer is loaded at `0x26dad3`, only after the options/program/package path has been entered, and its settings address is formed at `0x26dd6c`. When options exist, that address formation precedes the `numeric_inherit` test at `0x26dd75`, but this is outside the measured ordinary no-options path. A lazy caller closure alone therefore has no evidenced ordinary-send saving and is not the proposed repair.

## Adjacent code generation and narrow candidate

H carries `Option<PackageStart>` across construction and the conditional call to `Activation::cold_mut`. Its no-options path writes the None tag with `mov $0x2,%dil` at `0x26d26b`, then later tests that tag with `cmp $0x2,%dil` and `je` at `0x26d616` and `0x26d61a`. H also copies the 472-byte callee between stack slots through `memcpy` at `0x26d69c-0x26d6b4` before calling `push_activation`, whereas original e1 constructs into the final slot and calls `push_activation` directly at `0x26c3ec-0x26c3f7`. Original e1 merges default and package settings before writing the activation fields, visible at `0x26c16f-0x26c18b` and the subsequent constructor stores. These are concrete assembly differences, but their net cost cannot be inferred by adding local instruction counts because register allocation, stores, and libc-subtracted accounting also differ.

The smallest supported experiment is to move the unchanged executable-retention block immediately below `if let Some(start) = start { start.apply(&mut callee); }` in `dispatch/method_entry.rs`. This restores adjacency between construction and package application, and avoids keeping the package result live across the hit-only `cold_mut` call. `PackageStart::apply` only assigns settings, condition syntax, and optional trace mode into the local callee, so this move introduces no collection, Rexx callback, activity switch, or dictionary mutation between selecting the same method and retaining it. Retention still precedes activation flags, `push_activation`, trace entry, guard waiting, and user code, preserving replacement/removal/alias identity and GC behavior. The method-entry block move changes neither registry representation nor activation layout and keeps the existing comments and package-start timing contract intact.

The potential target is the package-result bookkeeping and its interference with constructor optimization, not elimination of the four necessary registry checks. The three visible tag instructions identify work that construction/application fusion might remove, but an unbuilt source rearrangement does not prove a three-instruction saving. In particular, the callee address still reaches an out-of-line `cold_mut` on registered hits, so the stack copy may remain after the move. No measured five-instruction repair is established by this read-only investigation, and speculative map redesign or broader cold-storage changes are not justified by it. The controller can accept or reject the one-block experiment after scoped identity/GC and directive-options controls, emitted-assembly inspection, and matched H-based measurements.

## Reproduction and ownership

The inspected method-entry, MethodRows, activation, and package-start files have no committed difference from H in the current f-owned tree. The decisive assembly can be reproduced from the immutable pins with the commands below, without building or executing either interpreter. The source contracts are in `method_rows.rs:29-35`, `dispatch/method_entry.rs:101-130`, `lib.rs:2697-2707`, `lib.rs:1893-1900`, and `activation.rs:871-924,964-966` at H. This report is the only file written by this investigation, and no index or HEAD change was made. Source/Git ownership remains with f, while whole-H profiles and the next repair decision remain with the controller.

```sh
objdump -d -C --no-show-raw-insn --start-address=0x26d26b --stop-address=0x26d2c8 /tmp/codex-p9d-t88-pins/8.8e2-perf-r1-rexx
objdump -d -C --no-show-raw-insn --start-address=0x26d4f3 --stop-address=0x26d52f /tmp/codex-p9d-t88-pins/8.8e2-perf-r1-rexx
objdump -d -C -l --no-show-raw-insn --start-address=0x26da91 --stop-address=0x26db1d /tmp/codex-p9d-t88-pins/8.8e2-perf-r1-rexx
objdump -d -C -l --no-show-raw-insn --start-address=0x26dd5c --stop-address=0x26ddd2 /tmp/codex-p9d-t88-pins/8.8e2-perf-r1-rexx
objdump -d -C -l --no-show-raw-insn --start-address=0x26d5f7 --stop-address=0x26d6c5 /tmp/codex-p9d-t88-pins/8.8e2-perf-r1-rexx
objdump -d -C --no-show-raw-insn --start-address=0x26c16f --stop-address=0x26c3fc /tmp/codex-p9d-t88-pins/8.8e1-rexx
```
