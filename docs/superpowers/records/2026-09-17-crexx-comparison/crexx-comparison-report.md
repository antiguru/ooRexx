# CREXX design comparison

Assembled by the controller from `crexx-compare`'s messages, because the harness
refused the agent's own write. Parts arrive one per message and are appended here
verbatim. Delivery truncated the first send after the opening of section 1, so
part 2 re-sends the macro table from the start.

## Provenance

`https://github.com/adesutherland/CREXX`, cloned to
`.../scratchpad/crexx-compare/CREXX`, branch `develop`, HEAD
`48ebc1f610a948a39972348379fd02fff4b156ad` (2026-09-03), `VERSION` =
`1.0.0-beta.3`. `df -h /tmp` before cloning: 63G total, 40G available; clone is
717M.

Built with `cmake -S CREXX -B build -DCMAKE_BUILD_TYPE=Release
-DBUILD_TESTING=OFF` then `cmake --build build -j 12`, gcc, exit 0. Configure
printed `cREXX product VM: rxvm -> rxtvm (GNU)` and `cREXX VM handler panel:
profile-20`.

Statement kinds are marked throughout: **[READ]** = read in their source, cited;
**[DOC]** = their documentation claims it; **[MEASURED HERE]** = ran on this
machine; **[THEIRS, UNVERIFIED]** = their figure, not reproduced, not comparable
to ours; **[INFER]** = inference, not a reading.

The agent ran no builds, tests or benchmarks of ours and wrote nothing into the
repository.

## Two corrections to the brief

**1. CREXX has an object model.** The brief said "no classes, no `~`, no
directives". The first part is wrong as of this checkout.

- **[READ]** `examples/ooBank.crexx` declares `BankAccount: class`, a `*:
  factory` constructor, typed instance attributes (`owner = .string`, `balance =
  .int`), and calls `account.deposit(50)` / `account.summary()`.
- **[READ]** `performance/NR-04A-RUNTIME-TYPE-DISPATCH-DESIGN.md` ("Current
  language contract") lists classes implementing interfaces, abstract and default
  interface methods, named factories, class-side `match` provider selection,
  `is`/`as` casts, and runtime type introspection. The same section then says:
  *"The documented current model has no class-superclass surface and explicitly
  does not implement interface inheritance."* The class graph is **flat** — no
  superclass chain, no metaclass, no mixins.
- **[MEASURED HERE]** `/bin/grep -c -F '~' compiler/rxcpbscn.re` -> `0`, same over
  `compiler/rxcpbgmr.y` -> `0`. There is no `~` operator; sends are dot notation.

The brief's instruction to assess candidates against our object model still
holds, for a narrower reason: they have objects with a flat, statically resolved
class graph, which is a far cheaper problem than ooRexx's.

**2. CREXX has neither `INTERPRET` nor `DROP`.** **[READ]**
`compiler/rxcpbscn.re:222` is `//  'INTERPRET' { RET(TK_INTERPRET); }` and `:210`
is `//  'DROP' { RET(TK_DROP); }`, both commented out in the scanner. This is the
same shape as the withdrawn bytecode-VM spike, except that here the missing
instructions are the language definition, and they are exactly what buys
section 3.

---

## 1. Dispatch

**Short answer: not reachable from safe Rust, and on this machine it is worth
about 12%. Do not plan around it.**

### What `rxtvm` is [READ]

`rxbvm` and `rxtvm` are the same C compiled twice.
`interpreter/CMakeLists.txt:327` gives `rxbvm` `NTHREADED=1`; `:333` builds
`rxtvm` without it, guarded by `CREXX_THREADED_VM_SUPPORTED`, with the comment at
`:330` — "Direct-threaded version. MSVC and unknown compilers deliberately do not
compile this GNU/Clang extension." `rxvm` is a copy of whichever was selected;
here `ls -la build/bin` shows `rxvm -> rxtvm`.

The entire difference is four macros in `interpreter/rxvmintp.h`:

| macro | NTHREADED (rxbvm), :357-:363 | threaded (rxtvm), :367-:376 |
|---|---|---|
| START_OF_INSTRUCTIONS | `CASE_START:; switch ((instructions)(pc->instruction.opcode)) {` | *empty* |
| START_INSTRUCTION(inst) | `case OP_##inst:` | `inst:` (a label) |
| VM_RESOLVE_SELECTED() | *empty* | `next_inst = next_pc->handler;` |
| VM_DISPATCH_TARGET() | `goto CASE_START` | `goto *next_inst` |

This is **direct** threading, not token threading: the handler address lives **in
the instruction cell**, so dispatch is one load plus one indirect jump with no
table indirection. `binutils/include/rxbin.h:96`-`:103` is the union that allows
it:

```c
typedef union bin_code {
    instruction_coding instruction;   /* { int opcode; int no_ops; } */
    void *handler; /* Process-local execution images only; never serialized. */
    double fconst;  rxinteger iconst;  char cconst;  size_t index;
} bin_code;
```

with `:107`-`:108` asserting `sizeof(bin_code) == 8`.

The rewrite happens once per module at first run. `interpreter/rxvmintp.c:5341`
`rxvm_prepare_execution_image` copies the module's canonical binary into a
process-local `execution_image` (`:5362`) and then overwrites each instruction
cell with `handler_address_map__[opcode]` (`:5397`-`:5400`). The canonical stream
is never mutated, so the `.rxbin` on disk stays portable and cross-platform.
`:6229`-`:6232` is the idempotent per-module driver, and its comment says why it
has to live inside `run()`: "address_map is only valid in this run() function" —
the label addresses are function-local, so the threading pass cannot be hoisted
out of the interpreter function.

The same pass also performs two in-place peephole substitutions
(`:5402`-`:5418`): `rxvm_private_r2_copyattr1_candidate` and
`rxvm_private_r1_relink_candidate` replace a recognised instruction with a
private fused handler. In the threaded build they write a private *label
address*; in the switch build they write a private *opcode*. So superinstruction
fusion is available to both engines.

**Nothing is decoded per step.** The `no_ops` field in the cell is read only by
the prepare pass (`:5367`-`:5372`) and by the disassembler. In the hot loop, each
handler advances by its own hard-coded arity: `VM_ADVANCE(3)` appears literally
in the handler text, e.g. `interpreter/rxvmhandlers_control.inc:1163`. There is
no operand-count decode, no dispatch table bounds check, and no opcode load in
the threaded build at all — the cell holds the jump target directly.

One consequence worth noting for us: because the handler knows its own arity
statically, the instruction stream can be variable-length at no per-step cost.
That property is what makes their 8-byte cell viable, and it is also what a Rust
`match` arm cannot reproduce uniformly — see part 4.

### Reachable from safe Rust? [INFER, mechanisms named]

No, and there is no near-miss.

- Computed goto has no Rust equivalent, safe or unsafe: labels are not
  first-class values.
- Guaranteed tail calls (`become`) are unstable, so the "one fn per opcode,
  tail-call the next" formulation is off the table on a stable toolchain.
- What *is* available in safe Rust is a function-pointer table, or handler
  pointers stored in the stream, which is "call threading". That buys the
  per-opcode indirect-branch site but pays a real call and return per VM
  instruction and forces all interpreter state across the boundary. CREXX pays
  exactly this whenever it outlines a handler: `rxvm_handler_state`
  (`interpreter/rxvmintp.c:5460`-`:5528`) is a 25-field snapshot struct built
  solely to carry `run()`'s locals across an outlined-handler edge.
- Our `unsafe` opt-ins are `crates/rexx-api/src/ffi.rs`,
  `crates/rexx-api/src/load.rs`, `crates/rexx-core/src/lib.rs`
  (`/bin/grep -rn "allow(unsafe_code)" --include=*.rs crates/`; pinned by
  `crates/rexx-core/tests/unsafe_sites.rs:89`-`:91`). **None is in `rexx-exec`**,
  and no grant would produce a computed goto anyway.

What we have, `match op` at `crates/rexx-exec/src/ir/drive.rs:538`, is the
`rxbvm` shape: LLVM lowers a dense enum match to a jump table, one indirect
branch through a table.

### [MEASURED HERE] The threading A/B, exactly what was run

**Binaries.** `build/bin/rxtvm` (through the `rxvm` symlink) and
`build/bin/rxbvm`, both produced by the single `cmake --build build -j 12`
invocation from the same sources, Release, gcc. The only intended difference is
`NTHREADED=1` on `rxbvm`. The two disassemblies were **not** diffed, so strictly
the measured difference is "everything `NTHREADED` gates", which per
`/bin/grep -n NTHREADED interpreter/rxvmintp.c` also includes some
`rxvm_handler_state` field selection and a sparse-dispatch variant, not only the
four macros in part 2.

**Program.** `probe/hot.crexx`, a 30,000,000-iteration `do while` loop whose body
compiles to exactly four VM instructions, `bgt` / `iadd` / `iadd` / `br`,
confirmed by reading `rxdas hot` output (addresses `0x000029`, `0x00002d`,
`0x000031`, `0x000035`). `probe/hot2.crexx` is identical but for one extra
`iadd r3,r3,r0` in the body, confirmed the same way.

**Run 1, whole-program instruction counts, `perf stat -e instructions`, five
interleaved pairs.** Three pairs from
`for r in 1 2 3; do for vm in rxvm rxbvm; do ...` plus two more from the
marginal-cost loop below, which also emits a `hot` count for each binary. Every
pair runs `rxvm` then `rxbvm` back to back in the same round.

- `rxtvm`: 2,975,110,828 / 2,974,657,866 / 2,974,722,153 / 2,974,975,301 / 2,975,378,721
- `rxbvm`: 3,364,994,248 / 3,364,928,435 / 3,365,380,626 / 3,365,206,944 / 3,365,936,474

Within-binary spread is 720,855 (0.024%) for `rxtvm` and 1,008,039 (0.030%) for
`rxbvm`. The between-binary gap is **13.1%**, roughly 400x the spread. Ratio
`rxbvm/rxtvm` = **1.131**.

**Run 2, marginal cost of one VM instruction, two interleaved pairs.**
`(hot2 - hot) / 30,000,000` per binary, which cancels startup and the rest of the
program:

- `rxtvm`: 27.989 and 28.008 host instructions per `iadd`
- `rxbvm`: 32.079 and 31.970

**Two pairs, not one.** Stated plainly because it was asked: the marginal-cost
figure rests on two rounds, the whole-program ratio on five.

**Run 3, wall clock, five interleaved rounds, `/usr/bin/time -f %e`.**

- `rxtvm`: 0.25 0.28 0.27 0.27 0.25 s
- `rxbvm`: 0.32 0.30 0.30 0.33 0.29 s

### What the layout-noise objection does and does not reach

The project's noise band is a **cycles** result: code placement moves cycles, it
does not move *retired instruction counts*, because retired instructions are a
property of the executed path and not of where the code sits. So the 1.131
instruction ratio is not a layout artifact, and the 0.024%/0.030% within-binary
reproducibility is the evidence that it is measuring a path difference. The
wall-clock ~11% **is** exposed to layout and should be read as agreeing in
direction only, not as an independent confirmation of the size.

**Headline as it should be stated: ~13% of retired instructions, 4.0 host
instructions out of ~32 per VM instruction.** The handler body costs the other
~28. The technique we cannot have is the small half; the part that costs 7x more
is value and operand representation, and there we are ahead of them (part 5).

**Falsification.** A Rust call-threading prototype over our `Op` stream beating
the `match` by more than a couple of percent on `rexxcps` would invalidate this
ranking. It needs no `unsafe`. Not run.

**Do not transfer their own ratio.** CREXX's perf programme runs on Apple Silicon
(`performance/DECISIONS.md:21`, "the complete first candidate panel on the Apple
M5"). This measurement is x86-64 on this machine. Indirect-branch prediction
differs enough between those cores that neither number licenses the other.

---

## 2. Instruction and register design

### Theirs [READ]

- **Register machine, but a register is a pointer.** `rxvmintp.h:466`:
  `#define REG_OP(n) current_locals[(pc+(n))->index]`. `current_locals` is
  `value **`, so a register is a **pointer to a 176-byte heap `value`** and every
  operand read is two dependent loads before the field access. `rxvmintp.h:474`
  then does `#define INT_VAL(vx) vx->int_value`.
- **Fixed 8-byte cells, variable-length instructions.** One instruction cell plus
  one cell per operand, read as `(pc+n)->index` (register number or const-pool
  offset), `(pc+n)->iconst`, or a const-pool lookup for floats and strings
  (`rxvmintp.h:466`-`:472`).
- **Opcode count.** `/bin/grep -c '^X(' binutils/include/rxops.h` -> **659**, of
  which `/bin/grep -c '^X(RESERVED_' binutils/include/rxops.h` -> **67** are
  reserved placeholders.
- **Operand shapes**, from
  `/bin/grep -o 'FMT_[A-Z_0-9]*' binutils/include/rxops.h | sort | uniq -c | sort -rn`:
  `FMT_R_R_R` 152, `FMT_EMPTY` 81, `FMT_R_R` 68, `FMT_R` 58, `FMT_R_R_I` 29, then
  a long tail. A further 50 ops carry a literal format string rather than an
  `FMT_` macro
  (`/bin/grep -c '^X([A-Z_0-9]*, *[0-9]*, *"' binutils/include/rxops.h` -> 50);
  these are the fused superinstructions, the widest nine operands (`"RRRRRRRRR"`).
- **Compare-and-branch is fused at emission, not by an optimiser pass.**
  [MEASURED HERE] `do while i <= n` compiles to a single `bgt lb_37,r0,r1`
  (`0x000029:0160`, "if op2>op3 then goto op1") under both `rxc` and `rxc -n`
  (their no-optimise flag; `rxc -h` lists `-n  No Optimising`). They also have
  `brtf lb_a,lb_b,r0`, one instruction covering both edges of a conditional,
  seen emitted for a loop whose condition was held in a variable
  (`probe/split.rxas`).
- **Hand-tiered handler placement.** `interpreter/rxvmhandlerpolicy.h:30` sets
  `CREXX_VM_HANDLER_PANEL 20`; from `:42` onward every opcode gets a tier
  (`HOT_05`, `HOT_10`, ... `MAXIMUM`, plus `NEVER`, `OWNER`, `RESERVED`).
  Handlers at or under the panel are expanded inline in `run()`; the rest are
  outlined behind the `rxvm_handler_state` snapshot. Hand-maintained I-cache
  management. The configure log printed `cREXX VM handler panel: profile-20`, so
  the build measured used this.

### Ours [READ]

- `crates/rexx-exec/src/ir.rs:56` asserts `size_of::<Op>() == 16`: fixed 16-byte
  ops, every op as wide as the widest variant.
- A register is an `ObjRef` in a contiguous frame:
  `self.roots.temp_at(registers, *lhs as usize)` (`drive.rs:1061`). **One load,
  and no dereference at all for a small integer**, because the integer is in the
  tag.
- Op variants, derived by
  `awk 'NR>=66 && NR<=269' crates/rexx-exec/src/ir.rs | /bin/grep -oE '^    [A-Z][A-Za-z0-9]*'`
  -> 43: `Clause TraceKeyword LoopHeaderValue LoopRun LoopNext TraceClause
  EvalExpr CallExpr PushArg TraceArgument CallArgs TraceFunction SelectCaseText
  WhenTest EndBranch EndWhen EnterWhen EnterOtherwise Const LoadConstant
  TraceLiteral Load TraceRead Arith Binary TraceOperator Prefix TracePrefix Store
  Say Signal Parse Return Queue Call CallNamed Message Expose Exec Escape Jump
  JumpUnless Condition`.
- Dispatch is `chunk.op_at_index(pc)`, a bounds-checked `self.ops.get(at as usize)`
  at `ir.rs:606`-`:608`, then `match op` at `drive.rs:538`.

### Density, concretely

[READ] `zw = zv + 1` compiles here to five ops (`golden_tests.rs:382`-`:387`):
`Clause`, `Load`, `LoadConstant`, `Arith`, `Store`. [MEASURED HERE] the
equivalent CREXX loop body carries no clause op at all. Our `Op::Clause { index,
end }` is a real per-clause op with region bookkeeping; theirs is a const-pool
`.srcstep` record keyed by address, costing nothing at run time (part 8,
candidate D).

### The actionable gap

Several of our ops are not native instructions: `Say { index }`,
`Parse { index }`, `Message { index }`, `Exec { index }`, `Call { index }`, and
`EvalExpr { index, slot, dst }` each carry an index back into the parsed
instruction tree and re-enter the AST interpreter. `EvalExpr`'s own doc says so
(`ir.rs:87`-`:92`, "still dispatched through `eval.rs` rather than reimplemented
here"). CREXX has no such escape hatch: all 659 opcodes are handlers. That is
candidate B.

The one *encoding* idea worth taking is compare-and-branch fusion, candidate C.

**Their 8-byte variable-length cell is not worth copying** [INFER]. Variable
length forces the arity into the handler, which is exactly what stops a Rust
`match` arm advancing the counter uniformly; and the density argument is weak
when our whole op already fits in a quarter of a cache line. The falsifying
observation would be a measured L1i or L1d miss rate on `rexxcps` attributable to
op-stream size, which, given this project's own layout-attribution result (dead
code alone moves cycles up to 4% per axis, `dispatch` spanning 5.9 pp,
non-monotonic in pad size), would need a padding control before it meant
anything.

**Survives `INTERPRET`/`DROP`/superclass/`~`?** Yes for compare-and-branch
fusion: it is a property of the operator's result type and nothing about dynamic
name resolution touches it. Yes for moving ops off the AST re-entry path, though
`Op::Exec` is precisely the `INTERPRET` op and is the one that must *stay* a
re-entry. The variable-length encoding is the one that would actively hurt: a
659-opcode typed ISA is unreachable for us (part 5), so we would be paying
variable-length decoding for a much smaller opcode set.

---

## 3. Value representation and typing

This is where the brief expected most transfer. The finding is the opposite:
**CREXX's typed instruction set is bought by not being Rexx**, and the two
commented-out scanner lines are the receipt.

### They do not reconcile "everything is a string" with typed ops, they removed the string case [READ + MEASURED HERE]

- **[READ]** There is no untyped `ADD`.
  `/bin/grep -n "^X(.*ADD_REG" binutils/include/rxops.h` returns only the `IADD`
  (int, `:40`-`:41`), `FADD` (float, `:325`-`:326`) and `DADD` (decimal,
  `:385`-`:386`) families.
- **[READ]** The `IADD` handler does **no type check at all**.
  `interpreter/rxvmhandlers_control.inc:1163` reads `op2RI`/`op3RI`, which expand
  through `rxvmintp.h:536`-`:538` and `:474` to
  `current_locals[(pc+n)->index]->int_value`, a raw field read with no flag test.
  `binutils/include/rxflags.h` carries no "int is valid" bit; there is nothing to
  test. The compiler is the only thing making it correct.
- **[READ]** The lattice is `compiler/rxcp_types.h:76`-`:78`: `TP_UNKNOWN,
  TP_VOID, TP_BOOLEAN, TP_INTEGER, TP_FLOAT, TP_DECIMAL, TP_STRING, TP_BINARY,
  TP_OBJECT, TP_REFERENCE`, with a static `promotion[10][10]` matrix in
  `compiler/rxcp_val_type.c` (the table immediately above `static ValueType
  node_type`). In it, `promotion[TP_STRING][TP_STRING]` is **`TP_FLOAT`**: two
  strings added give an IEEE double, not decimal arithmetic.
- **[DOC]** `docs/books/crexx_language_reference/variables.md:3`-`:5`: "Level B
  variables are typed. The compiler can infer many local variable types, but once
  a variable has a type, later assignments must be compatible with that type."
- **[MEASURED HERE]** That is enforced. `probe/retype.crexx`:

  ```
  options levelb
  a = 1
  a = "hello"
  say a
  ```

  `rxc retype` -> `Error in retype.crexx @ 3:5 - #BAD_CONVERSION: Value cannot be
  converted to the required type., "\"hello\""`, exit status 2. **A legal Rexx
  program does not compile.**

- **[MEASURED HERE]** Numeric semantics diverge from ANSI Rexx, using a
  runtime-unknown branch to defeat constant folding where needed
  (`probe/sem.crexx`, `probe/dec2.crexx`):

  | program | CREXX prints | ANSI/ooRexx |
  |---|---|---|
  | `a=0.1; b=0.2; say (a+b) = 0.3` | `0` | `1` |
  | `say 1 / 3` | `0` | `0.333333333` |
  | `n = 9007199254740993; say n + 1` | `9007199254740994` | same |

  The emitted `.rxas` says why. `a = 0.1` infers `.float`
  (`.meta "dec2.main.a"="b" ".float" r0`); the add emits `fadd r3,r0,0.2`, an
  IEEE double add; the comparison emits `feq r3,r3,0.3`. `1 / 3` is **integer**
  division and was constant-folded to `say "0"` in the assembler text.

- **[READ]** Conversions are explicit *instructions* the compiler inserts where
  the static type changes, never inside the arithmetic:
  `binutils/include/rxops.h:254` onward gives
  `BTOI/BTOD/BTOF/BTOS/ITOS/FTOS/ITOF/FTOI/FTOB/ITOB/STOB/STOF/STOI/STOD/DTOS/DTOI/DTOB/ITOD/FTOD/DTOF`,
  plus two-register forms (`rxops.h:321` is `X(STOI_REG_REG, 296, ...)`), plus
  `ASSERTTYPE_REG_STRING` at `:86` for the cases it cannot prove.

### Their value struct is far worse than ours [READ]

`interpreter/rxvalue.h:159`-`:191` holds, in one struct: `int_value`,
`float_value`, `decimal_value`, `string_value` plus three string metrics plus
three UTF-8 cache metrics, `binary_value` plus two lengths, native-payload ops
and flags, two reference cells, an object-type pointer, and four attribute
fields. `:193`-`:204` asserts the size at compile time: **176 bytes** (168 under
`NUTF8`).

Ours is `crates/rexx-core/src/handle.rs:47`, `pub struct ObjRef(u64)`, 2-bit
tagged, small integers to plus/minus 2^61 inline (`:43`-`:44`), byte strings to 7
bytes inline (`:23`), heap slot plus generation otherwise. A small-int add
touches no heap and costs one load per operand against their two.

### What this means for us

- **Their static types are unavailable.** ooRexx allows `a = 1; a = 'x'`, has
  `INTERPRET`, has `DROP`, and resolves routines and methods by name at run time.
  Their compiler can emit `IADD` because **none of that exists in their
  language**: `compiler/rxcpbscn.re:222` and `:210` are the two commented-out
  lines that pay for it.
- **The dynamic equivalent already exists here.** `Op::Arith` carries a per-site
  `hint` into `Chunk::hints`, tries `arith_small_int` on two `ObjRef`s and falls
  back to `arith_general`, permanently demoting a site that has fallen through
  once (`drive.rs:1072`-`:1094`, `ir.rs:393`-`:438`). That is a quickened typed op
  reached by feedback instead of by proof, and it is the right shape for a
  language that keeps `INTERPRET`.
- **The remaining delta is the guard, not the representation** [INFER]: they pay 0
  instructions to know the operands are ints; we pay a hint load, a branch and an
  `Option` test, a handful against their ~28 per instruction. Falsify by measuring
  the `tries_small_int` branch directly with callgrind on `rexxcps` rather than
  reasoning about it.

**Nothing in section 3 transfers.** The only adjacent idea worth anything is
candidate D, and it is small. Any future proposal to adopt typed opcodes here
should be checked against exactly one question: does it still work when the next
clause is `interpret v`?

---

## 4. Startup

The brief asks what they do at build time that we do at run time. **Their
ahead-of-time-linked variant is 150x worse than their ordinary one, and their
fast path is loading nothing.**

### What exists [READ]

`cpacker/rxcpack.c:133`-`:190` writes linked `.rxbin` files out as a C byte
array, `char library[] = { 0x.., ... };` followed by `size_t library_l = ...;`.
`interpreter/CMakeLists.txt:449`-`:465` generates `library.c` from `library`,
`classlib`, `classlib_native`, `rxfnsc`, `rxfnsg`, `rexxscript`; `:468`-`:473`
builds `rxvme` as `rxvmmain.c` + `library.c` with `LINK_CREXX_LIB=1`, and
`:483`-`:484` builds the switch-VM equivalent `rxbvme`. So `rxvme` is exactly the
"image linked at build time" product the brief was asking about.

### [MEASURED HERE] What it costs

Program: `options levelb / import rxfnsb / say 1`, compiled with `rxc` and
assembled with `rxas`, then run under `perf stat -e instructions`, three rounds
each:

| binary | instructions | wall |
|---|---|---|
| `rxvm` (= `rxtvm`, library loaded on demand) | 3,794,274 / 3,834,302 / 3,838,390 | 0.00 s |
| `rxbvm` (switch, same loading) | 3,768,447 / 3,729,457 / 4,038,179 | — |
| `rxvme` (**library linked into the executable**) | 581,548,393 / 579,999,720 / 574,055,829 | 0.18 / 0.20 / 0.21 s |

Binary sizes from `ls -la build/bin`: `rxbvm` 2,769,344 bytes, `rxvme`
10,131,320.

For calibration, the C++ ooRexx 5.3 oracle at
`/home/moritz/dev/repos/ooRexx/build/bin/rexx` on a one-line `say 1`, under the
standard `( ulimit -v 1048576; LD_LIBRARY_PATH=... )` wrapper, three runs:
**19,387,286 / 15,828,886 / 15,310,998** instructions.

`perf record` on `rxvme hello`, 587 samples, top symbols:

- `rxbin_reader_next_module` 32.04%
- `resolve_runtime_procedure` 25.62%
- `runtime_proc_matches_signature` 9.95%
- `rxvm_rebuild_interface_factory_registry` 7.80%
- `rxvm_rebuild_interface_method_registry` 6.17%

Confirmed the effect is not about whether the program *uses* the library: a
second probe calling `length("abc")` gave `rxvm` 3,827,116 and `rxvme`
533,774,171, the same shape.

### Reading it

- CREXX's 3.8M-instruction hello-world is roughly **a quarter of the oracle's**,
  and it is fast because a program importing nothing links nothing: there is no
  class library to materialise at all.
- The moment a class library *does* have to be materialised, which is precisely
  what `rxvme` does eagerly at every start, CREXX pays ~580M instructions, **about
  37x the oracle's image load**. The profile says where: reading modules out of
  the packed image, and rebuilding the procedure and interface registries. That is
  the same work our startup does.
- **So the technique that actually wins startup is the oracle's `rexx.img`**, a
  serialized *object* image that is mapped and fixed up rather than a byte image
  that is decoded and re-registered, and CREXX does not have it. Their evidence
  points away from "pack the library into the binary" and toward **"do not build
  what the program does not use."**
- Their format does support laziness structurally:
  `binutils/include/rxbin.h:67`-`:82` lists `RXBIN007_FEATURE_AUTOLOAD_HINTS`
  among the feature flags, and `rxc -h` carries `--autoload` / `--no-autoload`
  with autoload the default. So the ordinary `rxvm` path is demand-loading by
  design, not by accident.

### The caveat on the comparison to us

The 13.3x figure is the controller's, from the bench suite. It was not reproduced
and none of our binaries were run. This report does not record which descriptor it
is in, so **do not combine it arithmetically with the instruction counts above.**
What the numbers above support is a direction, not a ratio against us: eager
materialisation of a class library is expensive in CREXX too, and packing it into
the executable did not make it cheap.

**[INFER], falsifiable.** If a lazy-class-library experiment on our side moved
`startup` by less than half, the "eager materialisation is the cost" reading is
wrong and the cost is elsewhere: process setup, the parser, allocator warm-up.

**Survives `INTERPRET`/`DROP`/superclass/`~`?** The lazy-construction idea does,
and the superclass chain is the interesting part: ooRexx's class library is a
graph with inheritance, so lazily materialising one class may force its
superclasses, and the dependency closure is what decides whether laziness pays.
That is the thing to measure before planning it, not after.

---

## 5. Level G structured concurrency

Brief, and mostly a warning.

**Experimental by their own ledger.** [DOC] `concurrency/IMPLEMENTATION-STATUS.md`
carries status date 2026-08-16 and marks `task` procedures, `DO PARALLEL` and `DO
PARALLEL expr`, explicit `task target` expressions, transferable task methods and
`.taskwork` factory targets as "Implemented and directly tested; Level G only",
with Level B rejection covered by named negative tests (`gate_f_task_levelb`,
`gate_f_parallel_levelb`, `gate_f_task_target_levelb`). Their own status
vocabulary (`concurrency/README.md`, "Status vocabulary") is careful that
**implemented** is not **locally qualified** is not **portable** is not
**published initial** is not **stable**, and says explicitly: "An implemented
capability is not automatically portable, published or stable."

### The model [READ, from `concurrency/DECISIONS.md`]

- "Each execution owns its mutable globals, frames, registers, references,
  ordinary objects and runtime overlays. **Live VM storage never crosses an
  execution boundary.**"
- "Values cross as canonical `ChannelValue` documents and are materialized in
  receiver-owned storage. Transferable objects use an exact `to_channel` method
  and statically resolved `from_channel` factory."
- "Tasks are stateless invocations. Durable mutable state belongs to a
  single-owner service or actor identity whose accepted calls are serialized."
- "Large binary transfer uses explicit copy, move or immutable-seal states; it
  does not create writable shared Rexx memory."
- Deliberately absent: "OS thread creation or IDs, numeric worker affinity, locks,
  condition variables, atomics, fences, arbitrary writable shared memory, detached
  tasks, implicit replay of ambiguous remote work, or `async`/`await` suspended
  frames."
- Structured scope safety is enforced **statically, by the compiler**. [DOC] The
  status matrix row reads: "Negative compiler tests cover escaping pending
  results, mutation while pending, reference/exposed arguments, nested waits,
  reused scopes and invalid result types. Direct self-recursion is lowered as an
  ordinary synchronous procedure call; blocking waits across distinct task
  callables remain rejected."
- Layering is strict: the Level G surface lowers through public Level B
  pool/scope/task/completion/channel/value/endpoint classes, and those reach the
  VM through exactly five instructions, `chanopen`, `chanstart`, `chanwait`,
  `chancancel`, `chanclose`, with "no public RXPA task ABI or hidden
  native-payload contract".

### Why this is a different problem from ours [INFER]

ooRexx concurrency is share-**everything**: one object graph, `~start` returning a
Message object, guarded methods serialising access to an object's variable pool,
`reply` splitting an activation in two. Level G's entire safety argument rests on
two things we do not have and cannot get:

1. **No shared mutable object graph.** Their tasks cannot alias because live VM
   storage never crosses a boundary. ooRexx objects are shared by reference across
   activities by definition; that is what `GUARD` exists to manage.
2. **A static type system strong enough to prove a value transferable.**
   `from_channel` is "statically resolved"; the receiver rules are "statically
   enforced". Both are the same machinery as section 3, and both die the moment
   `INTERPRET` exists: an interpreted clause can name a task target the compiler
   never saw.

**Level G does not survive the arrival of `INTERPRET`, `DROP`, a superclass chain
or `~` dispatch.** `~start` alone breaks it, because it hands a Message object
back into the shared graph.

### What is worth taking, which is not a technique

Two habits, for the Phase 6 plan rather than Phase 6 code:

- **An explicit "deliberately absent" list.** Theirs names what the model will not
  expose, before anyone asks. Phase 6 will need one, because ooRexx's `GUARD` /
  `REPLY` / activity semantics are exactly where scope creep will come from, and
  "we did not decide that" reads identically to "we forgot" six weeks later.
- **The five-state status vocabulary.** implemented / locally qualified /
  portable / published initial / stable, each with its own evidence requirement,
  and the sentence that an implemented capability is not automatically any of the
  others. That maps directly onto this project's hollow-shell problem: it is the
  same distinction as "the method exists" versus "the method works", stated as a
  policy rather than caught by a reviewer.

Level G's mechanism is not worth Phase 6 planning time. Its decisions and status
matrix were read; no Level G program was built or run, so there is no measurement
of it at all.

---

## Ranked candidates

Ranked by (plausible win) x (probability it survives our object model) / cost.

### A. Per-send method cache, generation-guarded

Highest value, and the guard already exists in our tree.

**What it is [READ].** `interpreter/rxvmintp.h:100`-`:116`: `#define
RXVM_METHOD_CACHE_WAYS 2`, and `rxvm_dynamic_site_cache` holds a `uint64_t
generation` plus a union whose method arm is two `(const RxGraphTypeRef *type,
proc_runtime *target)` ways with a round-robin `uint32_t next_way`. The factory
arm is a separate single-entry cache with a three-state enum
(`rxvm_factory_cache_state`: UNKNOWN / DIRECT / GENERAL). Sites that need a cache
get a slot via `module::dynamic_site_cache_slots` (`:153`, "Instruction-word index
-> cache slot, or UINT32_MAX"), so the table is **dense over specialisable sites
rather than over the stream**, the same trick our `Chunk::hints` uses for
`Op::Arith`. Lookup is `rxvm_dynamic_cache_for_site` (`:174`-`:184`), three bounds
checks and an index.

**Evidence it works there.** [READ] the structure exists and is wired into the
module. [THEIRS, UNVERIFIED] `performance/NR-04A-RUNTIME-TYPE-DISPATCH-DESIGN.md`
records it as the selected design, "a two-way method-site cache, and factory
bucket/direct-target site caches complete the selected sealed-image hot shape",
and quotes "about 0.92-1.07 ns" for isolated descriptor support/dispatch. Their
figure, on Apple hardware, against a flat class graph, not reproduced.

**Cost here, lower than it looks, because the design was anticipated.**

- `crates/rexx-classes/src/class_graph.rs:64`-`:72` is `struct Behaviour { dict:
  MethodDict, version: u64 }`, and its comment states the version field "exists so
  a future per-call-site cache (D28's revisit condition) has something cheap to
  check rather than needing to be invented later against every mutating site".
  **Nothing reads it today.** The guard is built and unused, and every mutating
  site already bumps it.
- The shape to copy is the one we already run for routines: `ir.rs:441`-`:459`,
  `const CALL_SITE_CACHE: bool = true`, `struct CallSite { resolved:
  Cell<Option<Resolved>>, generation: Cell<u32> }`. Monomorphic,
  generation-guarded. The method version is the same pattern against
  `Behaviour::version`.
- What it replaces per send: `Interp::lookup` (`dispatch.rs:1805`) ->
  `MethodDict::slot` (`method_dict.rs:147`-`:158`) -> a hash-map probe. The
  uppercasing allocation has already been removed there (the code takes a borrow
  when the name has no lowercase byte); the hash and the compare have not.

**Does it survive `INTERPRET`, `DROP`, a superclass chain and `~` dispatch?**

- **`~` dispatch**: this candidate *is* the `~` path; the only one of the five
  that directly addresses it.
- **Superclass chain**: survives, and this is where we differ most from them.
  Their class graph is flat, so a type pointer identifies a method table outright.
  Ours does not: `MethodDict` is a *flattened* dictionary with `scope_orders`
  recording the cascade (`method_dict.rs:264`-`:278`), so the flattening has
  already happened and a behaviour handle plus its version is still a sound key.
  The cost is that `Behaviour::version` must be bumped by every `inherit`,
  `define`, `~setMethod` and mixin fold, which the comment says it already is, but
  which was not verified site by site and which is the first thing to check.
- **`~name:scope` override sends**: do not take the cache. `lookup_from_scope_at`
  is a different path (`dispatch.rs:1805` onward) and a cache keyed on receiver
  behaviour alone would answer the wrong method. Bypass it.
- **`INTERPRET`**: survives. An interpreted clause sends messages through the same
  `Interp::lookup`; it does not mutate the class graph except through operations
  that bump the version.
- **`DROP`**: irrelevant; it touches variables, not behaviours.
- **The real hazard is the object-own dictionary.** `own_method_entry`
  (`dispatch.rs`, just below `lookup`) reads `Body::Instance { own: Some(own), ..
  }`, a dictionary attached to a **single object**, not to its class, and consulted
  *first*. `Behaviour::version` cannot see a change to it. So the cache key must
  include "this receiver has no own dictionary", or the cache must be skipped
  whenever it does. Getting this wrong is a silently wrong method call, not a
  crash.
- `Op::Message { index }` carries no site id, so the op grows a `u16`, and `Op` is
  asserted at 16 bytes (`ir.rs:56`), so check the width before planning it rather
  than after.

**Falsifying observation.** Callgrind `bench-programs/dispatch.rex` and
`bench-programs/dispatchclass.rex`: if `MethodDict::slot` and its hash together
are under ~3% of instructions, this is not worth the correctness risk and the
candidate dies.

### B. Move `Op::Message` / `Say` / `Parse` / `Call` off the AST re-entry path

**What it is [READ].** CREXX has no fallback into a tree; all 659 opcodes are
handlers. We still have ops that carry an instruction index and re-enter `eval.rs`
or the clause interpreter: `ir.rs:87`-`:92` (`EvalExpr`, whose doc says "still
dispatched through `eval.rs` rather than reimplemented here"), `:211` (`Say`),
`:217` (`Parse`), `:243` (`Message`), and `drive.rs:1566`-`:1578` where
`Op::Message` fetches `InstructionKind::Message { term, value }` back out of the
parsed body and calls `exec_message`. The tree-walker was retired and
`Op::Generic` deleted, but this is the same residue under a different name: a
per-op bounds-checked descent into the parsed instruction, re-deriving what
compile time already knew.

**Evidence.** [READ] on both sides. **No measurement either way**, which is why it
ranks second rather than first. `rexxcps` is dominated by clause overhead, and
whether `Message` re-entry sits on that path is precisely the open question.

**Survives?** Yes, with one exclusion that should be written into any plan:
**`Op::Exec` is the `INTERPRET` op and must stay an AST re-entry**, because the
instruction it runs does not exist until run time. Everything else on the list is
statically known at compile time. Nothing about `DROP`, superclasses or `~` blocks
it.

**Falsifying observation.** If callgrind on `dispatch.rex` shows `exec_message`
and its callees already thin, with the cost sitting inside `Interp::lookup`, then
candidate A subsumes B and B should not be done separately.

### C. Fuse comparison with the branch that consumes it

**What it is.** [MEASURED HERE] `do while i <= n` in CREXX is one instruction,
`bgt lb_37,r0,r1`, emitted by `rxc` and by `rxc -n` alike. The same construct here
is three ops, `crates/rexx-exec/src/ir/golden_tests.rs:783`-`:791`, for `if 1 = 1
then say 'a' / else say 'b'`:

```
3: Binary op== lhs=0 rhs=1 dst=0
4: Condition index=0 reg=0 keyword=IF
5: JumpUnless reg=0 target=12
```

**Why it is sound.** With trace off, `Op::Condition`'s only job is to raise when
the value is not exactly `0` or `1` (`ir.rs:264`, and `ConditionKeyword::raiser`
returning `raised_if_not_logical` / `raised_when_not_logical`). A Rexx comparison
always answers exactly `"0"` or `"1"`, and ours are built by
`crates/rexx-exec/src/eval.rs:33`-`:39`, `logical(bool)` returning
`ObjRef::inline_byte(b'1')` or `b'0'`. So when the `IF`/`WHEN` expression's root is
a comparison operator, `Condition` is provably a no-op and the intermediate
register write is dead. A fused `CompareJump { op, lhs, rhs, target }` removes two
ops and one register write per test.

**Cost.** Bounded, and the trace complication is already solved: we emit a
different op stream per `ChunkTrace` (`compile_for_test` versus
`compile_for_test_under(..., traced())` in `golden_tests.rs`), so the fusion
conditions on trace-off exactly the way the existing trace ops already do. Under
trace, emit the unfused form and nothing changes.

**Survives?** Yes, and it is the cleanest of the five on this test. The property it
rests on is "a comparison operator's result is `0` or `1`", a fact about the
operator, not about name resolution. `~` sends inside the compared expressions are
unaffected: they are evaluated into registers before the fused op runs. An
`INTERPRET`ed clause compiles its own chunk and gets the same treatment.
Comparisons on objects still route through `Op::Binary` and still answer via
`logical`.

**Falsifying observation, run this before anything else.** Find a comparison
operator whose `Op::Binary` result is not `b"0"` or `b"1"`. `logical()` was
verified to produce only those two; every comparison operator's path to it was
**not** enumerated. If any comparison can answer something else, the fusion is
unsound and the candidate dies on the spot.

**Sizing, from their numbers and not ours.** One VM instruction costs ~28 host
instructions on `rxtvm` [MEASURED HERE]; removing two ops from a loop test is
worth about that twice, **in their VM**. Our per-op cost is different and
unmeasured. A direction, not a figure.

### D. Trace as address-keyed metadata rather than stream ops

**What it is.** [MEASURED HERE, via `rxdas`] CREXX's trace information is not
instructions. `.srcstep`, `.meta` and `.traceevent` appear throughout the
disassembly with **no `address:opcode` column**: they are constant-pool records
keyed by instruction position. The loop body is four instructions whether the
program is traced or not, and the `.traceevent` records sitting between them cost
nothing at run time.

We emit `TraceClause`, `TraceRead`, `TraceLiteral`, `TraceOperator`,
`TraceKeyword`, `TraceArgument` and `TraceFunction` as real ops, but we already
compile them out when trace is off (the untraced golden streams carry none), so
the **steady-state cost is already zero**. What we pay instead is recompilation
whenever the trace setting changes, plus a `ChunkTrace` field on every chunk and a
second compiled form to keep correct.

**Why it ranks low.** The win is not in the hot loop, because the hot loop already
has no trace ops. It is in not carrying two compiled forms of every chunk, a
maintenance and startup argument, not a throughput one.

**Falsifying observation.** If chunk recompilation on a `TRACE` change is not
measurable on any bench axis, and it should not be since `TRACE` is not in the
bench programs, this candidate is worth nothing and should be dropped rather than
scheduled.

### E. Hand-tiered handler inlining, their "handler panel". Do not adopt.

**What it is [READ].** `interpreter/rxvmhandlerpolicy.h:30` sets
`CREXX_VM_HANDLER_PANEL 20`; from `:42` onward every opcode carries a hotness
tier, and the panel selects how many are expanded inline in `run()`. The rest are
outlined behind the 25-field `rxvm_handler_state` snapshot
(`rxvmintp.c:5460`-`:5528`).

**[THEIRS, UNVERIFIED]** `performance/PERF3-13-WORKLIST.md`, "E5 Windows fallback
correction, 2026-08-13": a reconstructed owner that "polled the external word at
every instruction and invoked every handler through a generic outlined switch" was
"about 64% slower in `rxbvm` and 102% slower in `rxtvm`, while product executables
grew about 11%". Their number, their hardware, and for a design that **also** added
a per-instruction poll, so it does not isolate outlining at all.

**Why not.** This project has already measured what code layout does to its own
benchmarks: dead code alone moves cycles up to 4% per axis, `dispatch` spans 5.9
pp, and the effect is non-monotonic in pad size. A hand-maintained hotness table is
a layout experiment with no control, at a granularity Rust does not give us. It
would produce numbers we could not attribute, which is worse than no numbers.

### Not candidates

- **Direct threading**: unreachable from safe Rust, and worth ~13% of retired
  instructions where it *is* reachable.
- **Static types and typed opcodes**: requires deleting `INTERPRET`, `DROP` and
  dynamic retyping from the language, which is what CREXX did.
- **Their 176-byte `value`**: we are ahead by ~22x on the scalar representation and
  by one dependent load on every operand access.
- **Ahead-of-time-linked library image**: their own linked build is 150x slower to
  start than their ordinary one.

---

## The comparability problem, stated once

Every throughput number in this report that involves CREXX was taken on a program
their compiler type-checked end to end: no `~` send, no method lookup, no
class-library call, no `INTERPRET`, no `DROP`, no retyping of a variable, and
integer arithmetic proved statically rather than guarded at run time. Their own
benchmark directory is in the same position: `performance/` is largely Level B
numeric and string work on an Apple M5.

**No CREXX benchmark, theirs or measured here, is comparable to `rexxcps` or to
any of our bench axes.** This is the same failure the withdrawn bytecode-VM spike
hit, and the mechanism is worth naming so it is recognisable next time: the design
that produces the good number is fast *because* of what the language lacks, so the
number and the lack arrive together and the number gets quoted without it.

The one figure worth carrying forward is the **internal** A/B, `rxtvm` against
`rxbvm`, same source tree, same build invocation, same machine, same program,
because it isolates one technique and holds everything else fixed. Not their
absolute speed, and not any ratio between them and us.

## What could not be determined

- **Whether direct threading's ~13% here would be ~13% for us.** Our `match` arms
  are larger and fewer than their handlers, so the ratio of dispatch cost to
  handler-body cost is different. The A/B measures *their* ratio. The only way to
  get ours is the call-threading prototype, which needs no `unsafe`.
- **Whether the two binaries differ only in dispatch.** The disassemblies were not
  diffed. `#ifdef NTHREADED` also gates `rxvm_handler_state` field selection and a
  sparse-dispatch variant.
- **The size of candidates A, B and C on our side.** All three are sized from
  *their* per-instruction cost or from op counts, never from a measurement of
  ours. Each carries the callgrind run that would size it, and each should be
  sized before it is scheduled.
- **Which descriptor the "13.3x startup" and "1153 versus 531" figures are in.**
  They came from the brief, were not reproduced, and were not combined with any
  instruction count here.
- **Whether `rxvme`'s 580M-instruction startup is all library materialisation.**
  The profile attributes about 82% of *samples* to module reading and registry
  rebuilding, which is strong but is a sampled share, not an instruction share,
  and the two have come apart before on this project.
- **Whether every comparison operator in our tree routes through `logical()`.**
  This gates candidate C entirely and was not enumerated.
- **Whether `Behaviour::version` is bumped at every site that can change a
  flattened method dictionary.** Its own comment says it is, and that comment is
  the reason candidate A is cheap; the comment was read and the sites were not
  verified. If it is wrong, candidate A is a silently wrong method call rather
  than a slow one.
- **Level G's actual runtime performance.** Its decisions, status matrix and
  layering were read; no Level G program was built or run.

## Where the artifacts are

All under `scratchpad/crexx-compare/`:

- `CREXX/` the clone, HEAD `48ebc1f610a948a39972348379fd02fff4b156ad`
  (2026-09-03), `VERSION` 1.0.0-beta.3
- `build/` Release, gcc, `-DBUILD_TESTING=OFF`, exit 0; `build/bin` holds `rxc`,
  `rxas`, `rxlink`, `rxdas`, `rxtvm`, `rxbvm`, `rxvme`, `rxbvme` and the `rxvm`
  symlink
- `probe/` every probe program with its `.rxas` and `.rxbin`: `retype` (the
  `#BAD_CONVERSION` case), `sem` and `dec2` (the numeric divergences), `hot` and
  `hot2` (the threading A/B), `hotn` (`rxc -n`), `split` (the unfused
  compare/branch), `hello` and `hello2` (startup), `hello.rex` (the oracle
  calibration), and `rxvme.data` (the startup profile)
- `build.log`, `cmake-config.log`
