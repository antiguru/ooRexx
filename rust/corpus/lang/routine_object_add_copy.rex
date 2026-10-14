/* ~addRoutine of a library or internal row's Routine object, and of a copy
   of one: findRoutine answers the object added, and a call runs the row. */
p = .context~package
u = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleep')
c = u~copy
p~addRoutine('C1', c)
say 'internal copy' (p~findRoutine('c1') == c) (p~findRoutine('c1') == u) c1(0)
q = p~findRoutine('RXCALCSQRT')
p~addRoutine('SQ2', q)
say 'library' (p~findRoutine('sq2') == q) sq2(16)
d = q~copy
p~addRoutine('SQ3', d)
say 'library copy' (p~findRoutine('sq3') == d) (d == q) sq3(25)
r = .Package~new('r', 'nop')
p~addPublicRoutine('C2', u~copy)
r~addPackage(p)
p~addPublicRoutine('C2', u)
do i = 1 to 20; zj = 'filler' i; end
f = r~findRoutine('C2')
say 'importer' (f == u) (f == p~findRoutine('C2')) f~class~id f~call(0)
c = u~copy
p~addRoutine('C4', c)
.routines['C4'] = u
say 'written' (p~routines['C4'] == u) (p~routines['C4'] == c) (p~findRoutine('C4') == u)
signal on syntax
p~addRoutine('M1', .Method~new('m', 'nop'))
say 'method added'
exit
syntax: say 'method' condition('o')~code
::requires 'rxmath' LIBRARY
