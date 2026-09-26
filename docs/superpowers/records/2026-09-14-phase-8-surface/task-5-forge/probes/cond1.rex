d = CondInfo(40001)
call show d
d = CondInfo(88917, 'Conversion error')
call show d
say 'check' CondCheck(40001)
say 'decode' CondDecode(88917, 'bad thing')
say 'decode0' CondDecode(40001)
say 'display' CondDisplay(40001)
say 'none' CondNone()
d = CondUser('USER FOO', 1, .array~of('extra'), 'answer')
call show d
say 'after user'
exit
show: procedure
  use arg d
  if \d~isA(.Directory) then do; say 'not a directory:' d; return; end
  keys = 'ADDITIONAL CODE CONDITION DESCRIPTION ERRORTEXT INSTRUCTION MESSAGE PACKAGE POSITION PROGRAM PROPAGATED RC RESULT STACKFRAMES TRACEBACK'
  do ki = 1 to words(keys); k = word(keys, ki); if d[k] == .nil then iterate
    v = d[k]
    if v~isA(.Array) then v = 'Array('v~items')' v~makeString('l', ',')
    else if v~isA(.List) then v = 'List('v~items')'
    say ' ' k '=' v
  end
  return
::requires 'reach' LIBRARY
