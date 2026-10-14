p = .context~package
r = .Package~new('r', 'nop')
p~addPublicRoutine('C1', .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleep')~copy)
r~addPackage(p)
p~addPublicRoutine('C1', .Routine~loadExternalRoutine('y', 'LIBRARY rexxutil SysSleep'))
do i = 1 to 20; zj = 'filler' i; end
f = r~findRoutine('C1')
say 'importer' (f == p~findRoutine('C1')) f~class~id f~call(0) f~source~items
