#!/usr/bin/env python3
"""Classifies per-instruction costs (addrtsv.py output) into the report's classes.

usage: classify.py [--show CLASS] TSV...
First matching rule wins, so every instruction lands in exactly one class and
each file's classes sum to its total. Rules read the instruction text, the
function and the source position; they are applied to both engines, and a rule
naming only one engine's files matches nothing on the other.
"""
import re
import sys

WALK_HOIST = (0x13728F, 0x137469)  # walk_step_in_temps_frame's entry block: loads + spills


def rule(ir, addr, fn, f, ln, asm):
    fn = fn or ""
    # 4. missing caching: work re-derived on every execution that the oracle keeps on the node
    if ("slot_of" in fn or "HashMap<alloc::boxed::Box<[u8]>" in fn
            or ("assign_expr_target" in fn and 2734 <= ln <= 2737 and f.endswith("run.rs"))
            or ("assign_expr_target" in fn and f in {"src/token.rs", "slice/index.rs", "vec/mod.rs"}
                and ln in {98, 238, 1873})
            or fn.endswith("Interp>::literal") or "canonical_small_int" in fn
            or "inline_text_to_number" in fn or "walk_site_resolution" in fn
            or "small_int_operand" in fn or "spelled_int_arith" in fn):
        return "missing caching"
    # 7. extra work: D19's evaluation-depth bookkeeping
    if "enter_eval_node" in fn or (f == "src/eval.rs" and ln == 98):
        return "D19 depth"
    # 11. allocation and collection
    if (re.search(r"alloc_with|collect_now|heap::|Heap>|drop_glue|MemoryObject|operator new|"
                  r"MemorySegment|newObject|calloc|free", fn)):
        return "allocation"
    # 1. codegen: frame setup and teardown
    if re.match(r"^(push|pop|ret)\b", asm) or re.match(r"^(sub|add) \$0x[0-9a-f]+,%rsp$", asm):
        return "codegen"
    # 2. codegen: hoisted loads and spills at the walker's per-clause entry
    if "walk_step_in_temps_frame" in fn and WALK_HOIST[0] <= addr <= WALK_HOIST[1]:
        return "codegen"
    # error propagation: Result/Option/Flow niche tags, result.rs, and the walker's
    # ClauseOutcome<Result<Flow>> unpacking
    if (f == "src/result.rs" or re.search(r"\$0xffffffffffffff[f][def]\b", asm)
            or (f == "src/run.rs" and ln in {5028, 5037, 5041, 5049, 5052})
            or (f == "run/tree_walker.rs" and ln in {130, 146})):
        return "error propagation"
    # 3. codegen: spill/reload through the stack pointer (not lea: an out-pointer)
    if "(%rsp)" in asm and not asm.startswith(("lea", "call")):
        return "codegen"
    # 5. Rust-enforced checks
    if (f == "slice/index.rs" or (f == "src/option.rs" and re.match(r"^(test|j)", asm))
            or (f == "src/roots.rs" and ln in {349, 363}) or "panic" in asm):
        return "Rust checks"
    # 8. trace/debug gating and eager per-clause trace state
    if (f in {"src/trace.rs", "src/plan.rs", "src/bool.rs"}
            or (f == "src/run.rs" and (5455 <= ln <= 5465 or ln in {4945, 4949, 5081, 2667}
                                       or 8317 <= ln <= 8340))
            or (f == "src/activation.rs" and ln == 1096)
            or (f == "run/tree_walker.rs" and (ln == 138 or (ln == 0 and 0x13AC92 <= addr <= 0x13ACC4)))
            or (f == "src/eval.rs" and ln == 124)
            or (f.endswith("RexxActivation.hpp") and ln in {342, 356, 375, 380})
            or f.endswith("BitSet.hpp")):
        return "trace gating/state"
    # 9. rooting: the walker's temps Vec (len/cap/ptr at 0x2a0/0x290/0x298 off Interp), the
    #    oracle's expression stack and ProtectedObject
    if (re.search(r"0x2(90|98|a0)\(%r(12|13|14|15|si|di|bx)\)", asm)
            or "ExpressionStack" in f or "ProtectedObject" in f or "ProtectedBase" in fn):
        return "rooting"
    # 10. per-clause bookkeeping both engines do
    if (f == "src/clause.rs" or (f == "src/run.rs" and (4900 <= ln <= 4920 or ln == 4980))
            or (f == "vec/mod.rs" and ln == 3045)
            or (f == "src/activation.rs" and ln == 201) or f == "vec_deque/mod.rs"
            or (f == "mem/mod.rs" and ln in {967, 968})
            or (f.endswith("RexxActivation.cpp") and ln in {616, 639, 647, 651, 659})
            or f.endswith("RexxInstruction.hpp")):
        return "clause bookkeeping"
    # representation: decoding the tagged handle, copying inline text out of it, and the
    # alias/exposure tests every variable access makes
    if (f == "src/handle.rs" or fn.endswith("Interp>::render")
            or (f == "src/lib.rs" and ln == 5225) or (f == "src/roots.rs" and ln in {343, 344, 357, 358})):
        return "representation"
    return "work + dispatch"


CLASSES = ["codegen", "error propagation", "Rust checks", "missing caching", "D19 depth",
           "trace gating/state", "rooting", "clause bookkeeping", "allocation", "representation",
           "work + dispatch"]

show = None
args = sys.argv[1:]
if args and args[0] == "--show":
    show, args = args[1], args[2:]
print("file\t" + "\t".join(CLASSES) + "\ttotal")
for path in args:
    tot = dict.fromkeys(CLASSES, 0.0)
    for line in open(path):
        ir, addr, fn, f, ln, asm = line.rstrip("\n").split("\t", 5)
        c = rule(float(ir), int(addr, 16), fn, f, int(ln), asm)
        tot[c] += float(ir)
        if show == c:
            print(f"  {path.rsplit('/', 1)[-1]} {float(ir):6.2f} {fn[-40:]} {f}:{ln} {asm}")
    name = path.rsplit("/", 1)[-1].replace(".tsv", "")
    print(name + "\t" + "\t".join(f"{tot[c]:.1f}" for c in CLASSES) + f"\t{sum(tot.values()):.1f}")
