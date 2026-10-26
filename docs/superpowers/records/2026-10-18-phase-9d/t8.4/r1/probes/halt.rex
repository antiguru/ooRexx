m = .o~new~start("go")
x = m~result
say "after" x
::class o
::method go
  call sub
  return 1
sub:
  raise halt
