a = .Routine~loadExternalRoutine('x','LIBRARY REXX Filespec')
b = .Routine~loadExternalRoutine('y','LIBRARY REXX Filespec')
say 'ler twice same' (a == b)
f = .context~package~findRoutine('fs')
say 'directive same as ler' (f == a)
g = .context~package~findRoutine('fs2')
say 'two directives same' (f == g)
::routine fs external 'LIBRARY REXX Filespec'
::routine fs2 external 'LIBRARY REXX Filespec'
