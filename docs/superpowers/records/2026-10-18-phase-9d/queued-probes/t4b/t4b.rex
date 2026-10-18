d = .stringtable~new; d['A'] = 'return 1'
x = .object~enhanced(d)
say x~a
x~nosuch
