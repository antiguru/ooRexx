t = .T~new
say 'untrapped' t~rc('USER FOO', , .array~of(1,2), 'res') t~continue
t~rc('USER FOO'); say 'nores' var('result') result
t~rc('ERROR'); say 'error' var('result') result
call on user bar name handler
say 'callon' t~rc('USER BAR', , , 'r2')
say 'after callon'
signal on user baz
say 'signalon' t~rc('USER BAZ', , .array~of('a'), 'r3')
say 'not here'
exit
baz:
  o = condition('o')
  keys = 'ADDITIONAL CODE CONDITION DESCRIPTION ERRORTEXT INSTRUCTION MESSAGE PACKAGE POSITION PROGRAM PROPAGATED RC RESULT STACKFRAMES TRACEBACK'
  do ki = 1 to words(keys); k = word(keys, ki); if o[k] == .nil then iterate; if k <> 'STACKFRAMES' & k <> 'TRACEBACK' then say ' ' k '=' o[k]; end
  say 'caught' condition('c') '['condition('d')']' condition('i') condition('a')~items
  exit
handler:
  o = condition('o')
  keys = 'ADDITIONAL CODE CONDITION DESCRIPTION ERRORTEXT INSTRUCTION MESSAGE PACKAGE POSITION PROGRAM PROPAGATED RC RESULT STACKFRAMES TRACEBACK'
  do ki = 1 to words(keys); k = word(keys, ki); if o[k] == .nil then iterate; if k <> 'STACKFRAMES' & k <> 'TRACEBACK' then say ' ' k '=' o[k]; end
  say 'handler' condition('c') '['condition('d')']' condition('i')
  return 'fromhandler'
::class T
::attribute continue
::method rc external "LIBRARY orxmethod TestRaiseCondition"
