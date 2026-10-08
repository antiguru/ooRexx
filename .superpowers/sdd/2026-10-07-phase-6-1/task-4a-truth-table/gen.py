# Writes one probe program per value and context into the directory argv[1].
import os, sys
out = sys.argv[1]
values = [
 ("gt", "2 > 1"), ("dtrue", ".true"), ("eq", "1 = 2"), ("dfalse", ".false"),
 ("q1", "'1'"), ("i1", "1"), ("sum", "0+1"), ("cat", "'1'||''"), ("left", "left('12',1)"),
 ("str", "1~string"), ("tcopy", ".true~copy"), ("a1", ".array~of(1)"), ("q0", "'0'"),
 ("a0", ".array~of(0)"), ("banana", "'banana'"), ("sp1", "' 1'"), ("a12", ".array~of(1,2)"),
 ("s1", ".s1~new"), ("s0", ".s0~new"), ("sd", ".sd~new"),
]
classes = """
::class s1
::method string; return 1
::class s0
::method string; return 0
::class sd
"""
def prog(ctx, e):
    if ctx == "if": body = f"v = {e}\nif v then say 'T'; else say 'F'\n"
    elif ctx == "when": body = f"v = {e}\nselect; when v then say 'T'; otherwise say 'F'; end\n"
    elif ctx == "while": body = f"v = {e}\nt = 'F'\ndo while v; t = 'T'; leave; end\nsay t\n"
    elif ctx == "until": body = f"v = {e}\nn = 0\ndo until v; n = n + 1; if n > 1 then leave; end\nsay word('T F', n)\n"
    elif ctx == "not": body = f"v = {e}\nsay \\v\n"
    elif ctx == "and": body = f"v = {e}\nsay 1 & v\n"
    elif ctx == "list": body = f"v = {e}\nif 1, v then say 'T'; else say 'F'\n"
    elif ctx == "ifcmp": body = "o = .o~new\nif o = 1 then say 'T'; else say 'F'\n"
    elif ctx == "whencmp": body = "o = .o~new\nselect; when o = 1 then say 'T'; otherwise say 'F'; end\n"
    elif ctx == "whilecmp": body = "o = .o~new\nt = 'F'\ndo while o = 1; t = 'T'; leave; end\nsay t\n"
    elif ctx == "case": body = "c = .c~new\nselect case c; when 'x' then say 'T'; otherwise say 'F'; end\n"
    elif ctx == "doto": body = "n = 0\ndo i = .c~new to 3 for 3; n = n + 1; end\nsay n\n"
    elif ctx == "by": body = "n = 0\ndo i = .k~new to 5 by .b~new for 3; n = n + 1; end\nsay n\n"
    else:
        coll, meth = ctx.split("_")
        mk = {"arr": "k = .array~of(.c~new)", "lst": "k = .list~of(.c~new)", "tbl": "k = .table~new; k['k'] = .c~new"}[coll]
        body = f"{mk}\nsay k~{meth}(.c~new)\n"
    cls = classes
    if ctx in ("case",) or "_" in ctx:
        cls += f"::class c\n::method '=='; return {e}\n"
    if ctx in ("ifcmp", "whencmp", "whilecmp"):
        cls += f"::class o\n::method '='; return {e}\n"
    if ctx == "doto":
        cls += f"::class c\n::method '+'; return self\n::method '>'; return {e}\n"
    if ctx == "by":
        cls += f"::class k\n::method '+'; return self\n::method '>'; return 1 = 0\n::method '<'; return 1 = 1\n::class b\n::method '+'; return self\n::method '<'; return {e}\n"
    return body + "exit\n" + cls
ctxs = ["if","when","while","until","not","and","list","ifcmp","whencmp","whilecmp","case","doto","by","arr_hasItem","arr_index","lst_hasItem","lst_index","tbl_hasItem","tbl_index"]
for name, e in values:
    for c in ctxs:
        with open(os.path.join(out, f"{name}__{c}.rex"), "w") as f:
            f.write(prog(c, e))

