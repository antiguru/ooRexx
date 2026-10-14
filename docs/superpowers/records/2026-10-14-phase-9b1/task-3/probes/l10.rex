p = .context~package
c = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleep')~copy
signal on syntax
p~addRoutine('C1', c); say 'copy added' (p~findRoutine('c1') == c) c1(0)
exit
syntax: say 'copy' condition('o')~code
