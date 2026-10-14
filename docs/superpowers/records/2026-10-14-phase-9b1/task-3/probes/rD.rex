p = .context~package
u = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleep')
d = u~copy
w = .WeakReference~new(d)
p~addPublicRoutine('C3', d)
drop d
.routines['C3'] = u
r = .Package~new('r', 'nop')
do i = 1 to 20
  zj = 'filler' i
end
if w~value == .nil then say 'collected'
r~addPackage(p)
f = r~findRoutine('C3')
say (f == u) f~class~id f~call(0)
