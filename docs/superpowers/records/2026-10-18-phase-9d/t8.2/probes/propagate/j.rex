o = .c~new
say o~m
exit
::class c
::method m
  signal on syntax
  x = 1/0
syntax:
  call s
  return 1
s:
  interpret "call t"
  return
t:
  raise propagate
