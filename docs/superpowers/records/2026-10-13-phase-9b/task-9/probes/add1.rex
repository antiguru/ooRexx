signal on syntax
p = .context~package
p~addRoutine('FF2', p~findRoutine('ff'))
say 'added' ff2('n', '/a/b.c')
p~addRoutine('RR2', p~findRoutine('rr'))
say 'added' rr2(0)
exit
syntax: say 'syntax' condition('o')~code
::routine ff external 'LIBRARY REXX Filespec'
::routine rr external 'LIBRARY rexxutil SysSleep'
