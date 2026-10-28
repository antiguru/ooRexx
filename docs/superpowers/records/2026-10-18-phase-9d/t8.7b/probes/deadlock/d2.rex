o = .c~new
o~go
say 'after'
::class c
::method zz
  return 1
::method go
  m = self~start('zz')
  say 'waiting'
  m~result
