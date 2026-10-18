d = .stringtable~new; d["GO"] = "return self~items"
x = .array~enhanced(d, 3)
say x~go x~class~id
d = .stringtable~new; d["GO"] = "expose x; x = 3; self~setMethod('m', 'return 42', 'OBJECT'); return x self~m"
x = .string~enhanced(d, 'abc')
say x~go x~class~id
