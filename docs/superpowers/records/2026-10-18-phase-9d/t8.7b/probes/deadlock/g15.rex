.c~define('ZZ', .c~e)
o = .c~new
o~go
say 'after'
::class c
::constant e (.context~executable)
::method go
  m = self~start('zz')
  say 'waiting'
  m~result
