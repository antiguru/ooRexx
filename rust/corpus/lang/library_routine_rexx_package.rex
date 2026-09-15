/* A ::ROUTINE bound to an entry of the REXX package's routine table runs that
   routine under the directive's name, its Routine reports the REXX package,
   and loadExternalRoutine answers the entry, found without regard to case, or
   .nil for a name the table does not have. */
say 'fs' fs('N', '/a/b.c')
signal on syntax name bad
say fs('Z', 'x')
bad:
  c = condition('O')
  say 'bad' c~code '|' c~message
signal on syntax name none
say fs()
none:
  c = condition('O')
  say 'none' c~code '|' c~message
r = .context~package~findRoutine('FS')
say 'findRoutine' r~call('E', 'q.rs') r~package~name
say 'filespec' filespec('N', '/a/b.c')
loaded = .Routine~loadExternalRoutine('x', 'LIBRARY REXX filespec')
say 'loaded' loaded~class~id loaded~call('P', '/d/e.f') loaded~package~name
say 'nosuch' .Routine~loadExternalRoutine('x', 'LIBRARY REXX nosuch')
trace r
y = fs('N', '/g/h.i')
trace o
::requires 'pk.cls'
