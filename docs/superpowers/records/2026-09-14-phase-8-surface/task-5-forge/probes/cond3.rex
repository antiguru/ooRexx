signal on syntax
call RaiseKept 40001
exit
syntax:
  d = condition('o')
  do l over d~traceback; say '['l']'; end
  do f over d~stackframes; say f~type '|' f~name '|' f~line '|' f~executable~class '|' f~traceline; end
  keys = 'ADDITIONAL CODE CONDITION DESCRIPTION ERRORTEXT INSTRUCTION MESSAGE PACKAGE POSITION PROGRAM PROPAGATED RC RESULT STACKFRAMES TRACEBACK'
  do ki = 1 to words(keys); k = word(keys, ki); if d[k] == .nil then iterate; if k <> 'STACKFRAMES' & k <> 'TRACEBACK' then say k '=' d[k]; end
  exit
::requires 'reach' LIBRARY
