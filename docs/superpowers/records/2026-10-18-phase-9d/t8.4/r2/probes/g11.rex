.c~define('ZZ', .c~e)
o = .c~new
o~go
::class c
::constant e (.c~note(.context~executable))
::method note class
  say 'note ran'
  return arg(1)
::method go
  m = self~start('zz')
  call syssleep 0.3
  say 'go still holds'
  say 'go done'
