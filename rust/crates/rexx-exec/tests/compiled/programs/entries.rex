call rt
say .c~new~m
signal on syntax
say .c~new~boom
exit
syntax: say 'trapped' condition('o')~code
exit
::routine rt
  trace r
  say 'in rt'
  return
::class c
::method m
  trace i
  x = 1 + 1
  return x
::method boom
  trace i
  raise syntax 40.1
