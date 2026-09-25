"""The ruled assignment of BASE (d6dd60d90) dispatch.rs units to children,
by BASE line range, resolved to unit keys (which survive earlier commits'
line shifts). Impl blocks whose every member moves are named whole.

usage: plan.py BASE_DISPATCH_RS  -> writes plan.json beside this file
"""
import json, os, sys
import splitlib

RANGES = {
    "reqstr": [(2995, 3388)],
    "buffer": [(6399, 6675), (6689, 8226), (8773, 8828)],
    "construct": [(6349, 6397), (6677, 6687), (8228, 8676)],
    "class_protocol": [(3441, 3501), (3529, 3579), (3637, 3650), (3705, 3733),
                       (3831, 4580), (5491, 5501), (5891, 5902), (6241, 6347)],
    "object_protocol": [(3390, 3439), (3503, 3527), (3581, 3635), (3652, 3703),
                        (3751, 3829), (5534, 5889), (5904, 6239), (8678, 8707),
                        (8830, 8838)],
    "array": [(4582, 5188), (5424, 5489)],
}
# Whole native-table class blocks whose every row points into one child.
ROW_CLASSES = {
    "buffer": ["MutableBuffer"],
    "construct": ["Pointer", "WeakReference"],
}
units = splitlib.load_units(sys.argv[1])
plan = {}
for child, ranges in RANGES.items():
    keys = []
    impl_blocks = {}
    for u in units:
        if u["key"].startswith(("row ", "tests::")):
            continue
        if any(a <= u["first"] and u["last"] <= b for a, b in ranges):
            keys.append(u["key"])
        elif any(a <= u["first"] <= b or a <= u["last"] <= b for a, b in ranges):
            sys.exit(f"{child}: {u['key']} {u['first']}-{u['last']} straddles a range edge")
    plan[child] = {"keys": keys}
    plan[child]["rows"] = [u["key"] for u in units if u["key"].startswith("row NATIVE_METHODS/")
                           and u["key"].split("/")[1] in ROW_CLASSES.get(child, [])]
json.dump(plan, open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "plan.json"), "w"), indent=1)
for c, p in plan.items():
    print(c, len(p["keys"]), "units", len(p["rows"]), "rows")
