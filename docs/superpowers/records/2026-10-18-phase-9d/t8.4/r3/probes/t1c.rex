o = .c~new
m = o~start('hold')
call syssleep 0.3
o~zz
m~result
::class c
::method zz
  return
::method hold
  call syssleep 0.6
  say 'hold end'
::options trace r
