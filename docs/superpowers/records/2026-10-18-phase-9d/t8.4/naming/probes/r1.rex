m = .o~new~start('go')
say 'started'
x = m~result
say 'after' x
::class o
::method go
  say 'in go'
  y = 1 / 0
  return y
