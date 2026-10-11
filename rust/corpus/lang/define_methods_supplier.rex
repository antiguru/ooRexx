/* ~defineMethods, ~subclass's class methods and ~enhanced walk a table
   that is not string-keyed through its SUPPLIER, in supplier order. */
t = .table~new; t['M'] = 'return 11'
.k~defineMethods(t); say .k~new~m
.k~defineMethods(.array~of('return 12')); say .k~new~send('1')
.k~defineMethods(.s~new); say .k~new~q
r = .relation~new; r['R'] = 'return 14'
.k~defineMethods(r); say .k~new~r
u = .table~new; u['C'] = 'return 15'
c = .object~subclass('c', .class, u); say c~c
v = .table~new; v['E'] = 'return 16'
o = .object~enhanced(v); say o~e
w = .table~new; w['M'] = 'return 1'; w['m'] = 'return 2'
.k~defineMethods(w); say .k~new~m
x = .relation~new; x['Q'] = 'return 3'; x['Q'] = 'return 4'
.k~defineMethods(x); say .k~new~q
.k~defineMethods(.array~of('return 5', 'return 6')); say .k~new~send('2')
.k~defineMethods(.s2~new); say .k~new~z
y = .table~new; y['M1'] = .nil
.k~defineMethods(y); say .k~new~hasMethod('M1')
signal on syntax
.k~defineMethods(.object~new)
exit
syntax:
  say condition('O')['CODE'] condition('O')['MESSAGE']
::class k
::method m1
::class s
::method supplier; return .supplier~new(.array~of('return 13'), .array~of('Q'))
::class s2
::method supplier; return .t~new(.array~of('return 17'), .array~of('Z'))
::class t subclass supplier
::method item; return 'return 99'
