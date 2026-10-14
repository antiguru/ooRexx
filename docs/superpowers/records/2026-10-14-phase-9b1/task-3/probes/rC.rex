p = .context~package
u = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleep')
c = u~copy
p~addRoutine('C4', c)
.routines['C4'] = u
say 'routines' (p~routines['C4'] == u) (p~routines['C4'] == c) (p~findRoutine('C4') == u)
d = u~copy
p~addPublicRoutine('C3', d)
.routines['C3'] = u
say 'public' (p~publicRoutines['C3'] == d) (p~routines['C3'] == u) (p~findRoutine('C3') == u)
r = .Package~new('r', 'nop')
r~addPackage(p)
say 'importer' (r~findRoutine('C3') == d)
