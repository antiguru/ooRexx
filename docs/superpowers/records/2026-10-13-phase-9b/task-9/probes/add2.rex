p = .context~package
u = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleep')
p~addRoutine('S1', u)
say 'ler' s1(0) (p~findRoutine('s1') == u) (p~routines['S1'] == u)
q = .Package~new('q', "::requires 'rexxutil' library")
i = q~findRoutine('SysVersion')
p~addPublicRoutine('S2', i)
say 'imported' s2()~word(1) (p~findRoutine('s2') == i) (p~publicRoutines['S2'] == i)
f = .Routine~loadExternalRoutine('y', 'LIBRARY REXX Filespec')
p~addRoutine('F1', f)
say 'rexx' f1('n', '/a/b.c') (p~findRoutine('f1') == f)
z = .Package~new('z', "::routine zz external 'LIBRARY rexxutil SysSleep'")
w = .Routine~loadExternalRoutine('w', 'LIBRARY rexxutil SysSleep')
p~addRoutine('S3', w)
say 'before reflection' s3(0)
say 'zz' (z~findRoutine('zz') == w)
p~addRoutine('S4', w)
say 'after reflection' s4(0)
r = .Package~new('r', 'nop')
r~addPackage(p)
say 'importer' (r~findRoutine('S2') == i)
c = u~copy
signal on syntax
p~addRoutine('C1', c)
say 'copy added' (p~findRoutine('c1') == c)
exit
syntax: say 'copy' condition('o')~code
