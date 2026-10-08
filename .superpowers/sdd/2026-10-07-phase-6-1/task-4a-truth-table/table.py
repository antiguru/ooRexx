# Prints the Markdown table of a results directory written by run.sh.
import os, sys, re
res = sys.argv[1]
vals = ["gt","dtrue","eq","dfalse","q1","i1","sum","cat","left","str","tcopy","a1","q0","a0","banana","sp1","a12","s1","s0","sd"]
ctxs = ["if","when","while","until","not","and","list","ifcmp","whencmp","whilecmp","case","doto","by","arr_hasItem","arr_index","lst_hasItem","lst_index","tbl_hasItem","tbl_index"]
def cell(v, c):
    b = os.path.join(res, f"{v}__{c}")
    rc = open(b+".rc").read().strip()
    out = open(b+".out").read().strip().replace("\n", "/")
    err = open(b+".err").read()
    m = re.findall(r"^Error (\d+\.\d+):", err, re.M)
    if rc == "0" and not m: return out
    return (out + " " if out else "") + (m[-1] if m else "") + f" rc{rc}"
print("| value | " + " | ".join(ctxs) + " |")
print("|---" * (len(ctxs)+1) + "|")
for v in vals:
    print(f"| {v} | " + " | ".join(cell(v,c) for c in ctxs) + " |")
