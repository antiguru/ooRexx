#!/usr/bin/env python3
"""Compare every function-pointer slot's signature in layout.rs against the frozen header."""
import re, sys

HEADER = "/home/moritz/dev/repos/ooRexx-rust-rewrite/api/oorexxapi.h"
LAYOUT = "/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-api/src/layout.rs"
NAMES = ["RexxInstanceInterface", "RexxThreadInterface", "MethodContextInterface",
         "CallContextInterface", "ExitContextInterface", "IORedirectorInterface"]

CMAP = {
    "RexxInstance*": "*mut RexxInstance_", "RexxThreadContext*": "*mut RexxThreadContext_",
    "RexxThreadContext**": "*mut *mut RexxThreadContext_",
    "RexxMethodContext*": "*mut RexxMethodContext_", "RexxCallContext*": "*mut RexxCallContext_",
    "RexxExitContext*": "*mut RexxExitContext_", "RexxIORedirectorContext*": "*mut RexxIORedirectorContext_",
    "size_t": "usize", "int": "c_int", "int64_t": "i64", "uint64_t": "u64", "int32_t": "i32", "uint32_t": "u32",
    "intptr_t": "isize", "uintptr_t": "usize", "double": "f64", "float": "f32",
    "constchar*": "*const c_char", "CSTRING*": "*mut CSTRING", "size_t*": "*mut usize",
    "wholenumber_t*": "*mut wholenumber_t", "stringsize_t*": "*mut stringsize_t", "logical_t*": "*mut logical_t",
    "int64_t*": "*mut i64", "uint64_t*": "*mut u64", "int32_t*": "*mut i32", "uint32_t*": "*mut u32",
    "intptr_t*": "*mut isize", "uintptr_t*": "*mut usize", "double*": "*mut f64",
    "RexxCondition*": "*mut RexxCondition", "ValueDescriptor*": "*mut ValueDescriptor",
    "RexxPackageEntry*": "*mut RexxPackageEntry",
}

def norm_c(t):
    toks = t.split()
    if len(toks) >= 2 and re.fullmatch(r"[A-Za-z_]\w*", toks[-1]) and toks[-2] not in ("const", "unsigned", "struct") and not toks[-1] in ("int","float","double","size_t"):
        toks = toks[:-1]
    t = re.sub(r"\s+", "", " ".join(toks))
    return CMAP.get(t, t)

def parse_header():
    text = open(HEADER).read().splitlines()
    out = {}
    for name in NAMES:
        end = next(i for i, l in enumerate(text) if l.strip() == "} %s;" % name)
        start = max(i for i in range(end) if text[i].strip() == "typedef struct")
        members = []
        for l in text[start + 1:end]:
            code = l.split("//")[0].strip()
            if not code or code == "{":
                continue
            m = re.match(r"(.+?)\(\s*RexxEntry\s*\*\s*(\w+)\s*\)\s*\((.*)\)\s*;", code)
            if m:
                ret, fname, args = m.groups()
                args = [norm_c(a) for a in args.split(",")] if args.strip() else []
                r = norm_c(ret)
                members.append((fname, "call", args, None if r == "void" else r))
            else:
                m = re.match(r"(.+?)\s+(\w+)\s*;", code)
                ty, fname = m.groups()
                members.append((fname, "value", norm_c(ty), None))
        out[name] = members
    return out

def parse_layout():
    text = open(LAYOUT).read()
    out = {}
    for m in re.finditer(r"interface!\s*\{(.*?)\n\}", text, re.S):
        body = m.group(1)
        hm = re.search(r"(\w+),\s*(populated|unpopulated)\s*\{(.*)\}", body, re.S)
        name, _, fields = hm.groups()
        members = []
        for fm in re.finditer(r"(\w+):\s*\{\s*(value|call)(.*?)\},?\n", fields, re.S):
            fname, kind, rest = fm.groups()
            rest = rest.strip()
            if kind == "value":
                ty = rest.split("=")[0].strip()
                members.append((fname, "value", ty, None))
            else:
                am = re.match(r"\((.*?)\)\s*(?:->\s*(.+))?$", rest, re.S)
                args, ret = am.groups()
                args = [re.sub(r"\s+", " ", a.strip()) for a in args.split(",")] if args.strip() else []
                members.append((fname, "call", args, ret.strip() if ret else None))
        out[name] = members
    return out

h, r = parse_header(), parse_layout()
bad = 0
for name in NAMES:
    hs, rs = h[name], r[name]
    if [x[0] for x in hs] != [x[0] for x in rs]:
        print("NAME ORDER DIFFERS", name); bad += 1
    for hm, rm in zip(hs, rs):
        if hm != rm:
            print(f"{name}.{hm[0]}: header {hm[1:]} vs rust {rm[1:]}"); bad += 1
    print(f"{name}: {len(hs)} members compared")
print("mismatches:", bad)
sys.exit(1 if bad else 0)
