import sys, os
d = sys.argv[1]
exprs = [")", "]", "a[1]]", "1 2 3)", "a b )", "(a))", "a +", "a ||", "a || || b", "a + * b",
 "**2", "~a", "a %% b", "a \\(1 = 2)", "()", "\\", "(a", "((a)", "f(a b", "a[", "a[1", "(a[1",
 "a~", "a~~", "a~b~", ".array~of(1,2)~[1]", "a[1)", "a(1]", "a [1]", "a[1] [2]", "a += 1",
 ">1", ">'&1'", "> a.b", "1 + -", "a~m:1", "a~m(1", "f(1,\n  (2", "1 :: 2", "a = = 1", "a | ",
 "'x' '80'x +", "+", "a ==", "ns:1"]
with open(os.path.join(d, "index.tsv"), "w") as idx:
    for i, e in enumerate(exprs):
        name = "e%02d" % i
        with open(os.path.join(d, name + ".rex"), "wb") as f:
            f.write(b"r = " + e.encode() + b"\n")
        idx.write(name + "\t" + e.replace("\n", "\\n") + "\n")
