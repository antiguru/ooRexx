"""LINES:a-b covering the units FIRST through LAST of a file, each unit's
span as splitlib takes it (comments directly above included).

usage: span_of.py FILE FIRST_KEY LAST_KEY
"""
import sys, splitlib
path, k1, k2 = sys.argv[1:4]
lines = open(path).read().split("\n")[:-1]
u = {x["key"]: x for x in splitlib.load_units(path)}
a, _ = splitlib.span(lines, u[k1]); _, b = splitlib.span(lines, u[k2])
print(f"LINES:{a + 1}-{b + 1}")
