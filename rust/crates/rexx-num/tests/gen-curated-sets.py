#!/usr/bin/env python3
"""Regenerates the curated differential case sets.

The random sets come from `cargo run -p rexx-num --bin gen-cases -- <seed> <n>`,
which is seeded and therefore reproducible on its own. These curated sets were
originally built by ad-hoc scripts that lived only in a session scratchpad, so
they are captured here — the value lists encode which shapes actually find
bugs, and rebuilding them from nothing costs a multi-minute oracle run each.

`fmt` and `fmt2` are RECONSTRUCTIONS, not byte-exact copies: they emit 1800
and 6720 cases where the scratchpad originals had 1116 and 7080. The argument
lists were re-derived rather than recovered, so the coverage overlaps heavily
but is not identical. Both reconstructions were run against the oracle and
sit at 0 divergences, so they are sound sets — they are simply not the same
sets, and no claim should be made that a pass here reproduces a pass there.
`fmt3` and `fmtedge` were written from this file and do reproduce byte for
byte; that was checked with `diff`.

Usage:  python3 gen-curated-sets.py <name> > cases.txt
        names: addsub addsub2 muldiv md2 pow cmp fmt fmt2

Then, from the repo root:
    build/bin/rexx rust/crates/rexx-num/tests/data-addsub-oracle.rex cases.txt
for arithmetic and comparison, or data-format-oracle.rex for fmt/fmt2.
"""
import sys

ARITH_A = ["0","1","-1","5","-5","9","10","99","100","-100","0.1","0.5","-0.5","1.5","1.50",
           "0.05","999999999","1000000000","123456789","987654321","1e5","1e-5","1e9","1e-9",
           "0.000001","12.3400","1.000000001","9.999999995","-0.000500","2.5","-2.5","1e18","1e-18"]
ARITH_B = ["7","-7","0.007","70000","7.0000007","6.66666665","3.5","-3.5","0.25","2500000",
           "8e7","8e-7","1e17","1e-17","999999999999","0.999999999","111111111","1.9999999995",
           "-0.0000001","54321.12345","1e-3","5e-4","0.0001","1000001","9999999999999999","-0"]
MULDIV_A = ["0","1","-1","2","-2","3","7","-7","10","100","0.5","-0.5","0.1","0.25","1.5","-1.5",
            "1.50","999999999","1000000000","123456789","1e5","1e-5","1e9","1e-9","3.14159",
            "2.5","-2.5","0.001","7.7","1e18","1e-18","6.66666665","0.0000001"]
POW_BASES = ["0","1","-1","2","-2","3","10","0.5","-0.5","1.5","1.50","0.1","7","-7","100",
             "1e5","1e-5","123456789","3.14159","0.001","2.5","9.99999999","1e100","1e-100"]
POW_EXPS = ["0","1","-1","2","-2","3","-3","10","-10","20","100","-100","1e2","2.5","0.5",
            "1e10","999999999","-999999999","1000000000","7","-7","33"]
CMP_VALS = ["1","1.0","1.00","01","0","-0","0.0","2","-2","1e2","100","100.0","0.1",".1",
            "1e-2","0.01"," 1","1 ","abc","ABC","a","b","","  ","1a","-1","+1","1E2",
            "999999999","1000000000","0.999999999","1.000000001","3.14","3.140"]
CMP_OPS = ["=","<",">","<=",">=","\\=","<>","><","==","\\==","<<",">>","<<=",">>="]
FMT_NUMS = ["3.14159","-3.14159","0","-0","1","-1","12345.6789","-12345.6789","0.000012345",
            "1e10","1e-10","1e100","1e-100","999999999","1000000000","1.5","2.5","-2.5","0.5",
            "-0.5","100","0.001","123456789.987654321","9.9999995","0.99999995","1e5","-1e5",
            "0.10","1.00","10.50"]

def binary(vals, ops, digits):
    return [f"{d}|{a}|{op}|{b}" for d in digits for a in vals for b in vals for op in ops]

def fmt(digits, befores, afters, expps, expts, places):
    out = []
    for d in digits:
        for n in FMT_NUMS:
            out += [f"{d}|TRUNC|{n}|{a}|||" for a in places]
            out += [f"{d}|FORMAT|{n}|{b}|{a}||" for b in befores for a in afters]
            out += [f"{d}|FORMAT|{n}|||{p}|{t}" for p in expps for t in expts]
    return out

FMT3_NUMS = ["7.25","-7.25","9.5","10.5","99.5","-99.5","12","123","1234",
             "1e0","1e1","1e2","1e3","5e0","5e1","5e2","0.0999","0.00999","9.99e-1",
             "1.001e2","-1.001e2","8","-8","80","800","8000","0.25","-0.25",
             "1e-1","1e-2","1e-3","1e-9","0.000000001","0.0001234","2e-7",
             "999.999","1000.0001","77777777.7","6.02e23","1.6e-19","0e0"]

def fmt3():
    """Independent third set: shares no values with `fmt`/`fmt2`, and reaches
    two things neither of them does.

    A digits field ending in `E` runs the case under NUMERIC FORM ENGINEERING;
    the earlier sets were SCIENTIFIC only, so the engineering path went
    untested even though it is what collapses a nonzero adjusted exponent to a
    displayed 0. The last family passes before/after *together with*
    expp/expt -- `fmt()` emits those two argument families separately, so the
    interaction where expp blank-pads an otherwise plain result was never
    exercised. Values cluster near adjusted exponent 0, 1 and 2 for the same
    reason. 15 of these differ between the two forms without erroring.
    """
    out = []
    for d in ["5", "9", "5E", "9E"]:
        for n in FMT3_NUMS:
            out += [f"{d}|TRUNC|{n}|{p}|||" for p in ["","0","1","2","5"]]
            out += [f"{d}|FORMAT|{n}|{b}|{a}||"
                    for b in ["","0","1","3","6"] for a in ["","0","1","3"]]
            out += [f"{d}|FORMAT|{n}|||{p}|{t}"
                    for p in ["","0","1","2","5"] for t in ["","0","1","2","4"]]
            out += [f"{d}|FORMAT|{n}|{b}|{a}|{p}|{t}"
                    for b in ["","2"] for a in ["","3"]
                    for p in ["","0","2"] for t in ["","0"]]
    return out

def fmtedge():
    """Exponent extremes, plus values that are not numbers at all.

    Two shapes here will fill a disk if you widen them carelessly, because
    both are faithful to the interpreter rather than wrong. `expp` of 0
    suppresses exponential form, so `format(1e999999999,,,0)` really is a
    billion plain digits; and TRUNC never uses exponential form at all, so
    TRUNC of any huge value is equally large. Every FORMAT argument set below
    therefore keeps exponential form, and TRUNC is confined to values whose
    plain form is short.

    The unparseable values are the point of the set: they are what showed the
    harness had been asserting the wrong error number for them.
    """
    fmt_args = ["|||", "0|||", "|9||", "||9|0", "9|9|9|9", "|||0", "|||1",
                "||12|", "3|3|12|4", "||1|"]
    big = ["1e999999999", "-1e999999999", "1e-999999999", "9.99999999e999999998",
           "1e999999998", "-9.9e-999999999", "1e300", "1e-300"]
    small = ["1e300", "1e-300", "3.9", "0.000012345", "-7.25", "1e20", "1e-20", "0"]
    out = []
    for n in big:
        for d in ["1", "9", "15", "20", "9E", "15E"]:
            out += [f"{d}|FORMAT|{n}|{a}" for a in fmt_args]
    for n in small:
        for d in ["1", "9", "15", "20"]:
            out += [f"{d}|TRUNC|{n}|{p}|||" for p in ["", "0", "1", "5", "20"]]
    return out

def fmtcarry():
    """The exponent-width check on the far side of a rounding carry.

    `format(9.996E99,,0,2)` raises 93 in the interpreter: rounding to zero
    decimals carries, the exponent goes from 99 to 100, and three digits no
    longer fit the two-wide `expp` field. A version that checks the width
    only *before* the carry answers `1E+100` instead, and 21,296 existing
    FORMAT cases all missed it, because the successful-render path agrees at
    the same boundary — only the error path diverges.

    So this set is deliberately narrow: mantissas that carry into an extra
    integer digit, at exponents just below a power-of-ten boundary where the
    carry widens the exponent too, crossed with `expp` values on either side
    of the width that results.
    """
    carriers = ["9.996", "9.9996", "9.99996", "9.5", "9.95", "9.999999999",
                "8.996", "9999999999.6", "99999999.96", "9.996", "1.996"]
    exps = ["", "E9", "E99", "E999", "E-9", "E-99", "E-999", "E10", "E100"]
    out = []
    for d in ["9", "15", "9E", "15E"]:
        for c in carriers:
            for e in exps:
                n = c + e
                for after in ["", "0", "1", "2"]:
                    for expp in ["", "1", "2", "3", "4"]:
                        out.append(f"{d}|FORMAT|{n}||{after}|{expp}|")
                        out.append(f"{d}|FORMAT|{n}||{after}|{expp}|10")
    return out

SIGNBLANK_A = ["+ 3","- 3","+  3","-   3","  + 3  ","+ .5","- .5","+ 3.","+ 0","- 0",
               "+ 1e2","- 1e-2","+ 12345678901","+\t3","\t+ 3\t","3\t","+ 3.14",
               "+ 999999999","- 0.000001","+ 3 e2","3 4","+ - 3","++ 3","3e 2","3e+ 2",
               "+ ","+","- .","+ abc"]

def signblank():
    """Blanks between a sign and its digits, and blank/tab edge handling.

    `numberStringScan` (NumberStringClass.cpp:1289-1295) skips blanks and
    tabs after a sign, so `'+ 3'` is the number 3 -- legal there, at either
    end, and nowhere else. None of the other eleven sets can see this: no
    generator above emits a blank after a sign, which is how a parser that
    rejected `'+ 3'` outright sat at 0 divergences across all 126,048 of
    their cases. The invalid spellings (`'+ 3 e2'`, `'3 4'`, `'++ 3'`, a
    bare or blank-followed sign) matter as much as the valid ones -- both
    directions were wrong before the fix. `**` is deliberately absent from
    the operator list: a non-numeric power operand raises through a
    different path than the <E41> this harness models for the other
    operators.
    """
    ops = ["+","-","*","/","%","//","=","==","<","<<"]
    return [f"{d}|{a}|{op}|{b}"
            for d in [3, 9] for a in SIGNBLANK_A for op in ops
            for b in ["0","2","+ 2","- 1.5"]]

SETS = {
    "addsub":  lambda: binary(ARITH_A, ["+","-"], [1,3,9,15]),
    "addsub2": lambda: binary(ARITH_B, ["+","-"], [2,4,6,7,11,20]),
    "muldiv":  lambda: binary(MULDIV_A, ["*","/","%","//"], [1,3,9,15]),
    "md2":     lambda: binary(ARITH_B + ["13","17","0"], ["*","/","%","//"], [2,4,6,7,11,20]),
    "pow":     lambda: [f"{d}|{a}|**|{b}" for d in [1,3,9,15] for a in POW_BASES for b in POW_EXPS],
    "cmp":     lambda: binary(CMP_VALS, CMP_OPS, [1,9]),
    "fmt":     lambda: fmt([5,9], ["","1","4","10"], ["","0","2","4"], ["0","2","4"], ["","0","2"], ["","0","1","2","3"]),
    "fmt2":    lambda: fmt([1,3,9,15], ["","0","1","2","6","12"], ["","0","1","3","8"],
                           ["","0","1","2","5"], ["","0","1","4"], ["","0","1","2","5","9"]),
    "fmt3":    fmt3,
    "fmtedge": fmtedge,
    "fmtcarry": fmtcarry,
    "signblank": signblank,
}

if __name__ == "__main__":
    if len(sys.argv) != 2 or sys.argv[1] not in SETS:
        sys.exit(f"usage: {sys.argv[0]} <{'|'.join(SETS)}>")
    print("\n".join(SETS[sys.argv[1]]()))
