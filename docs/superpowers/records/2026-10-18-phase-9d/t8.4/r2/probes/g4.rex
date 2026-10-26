.c~define('ZZ', .c~e)
o = .c~new
m = o~start('hold')
call syssleep 0.3
o~zz
say 'zz returned'
m~result
::class c
::constant e (.c~note(.context~executable))
::method note class
  say 'note ran'
  return arg(1)
::method hold
  say 'hold start'
  call syssleep 1
  say 'hold end'
