m = .t~new~start('boom')
m~wait
call SysSleep 0.3
c = m~errorCondition
signal on syntax name s1
say 'items' c~items
s1: if symbol('RC') == 'VAR' then say 'items refused' condition('O')~code
say 'copy items' c~copy~items
::class t
::method boom unguarded
  return 1/0
