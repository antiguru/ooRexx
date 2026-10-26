say .c~k
.c~define('ZZ', .c~e)
say 'defined'
signal on syntax name s1
say .c~new~zz
s1: say condition('o')~code
call sub
exit
sub:
  .c~new~zz
  say 'as instruction' var('result')
  return
::class c
::constant k (.c~note)
::constant e (.context~executable)
::method note class
  say 'note ran' self
  return 7
