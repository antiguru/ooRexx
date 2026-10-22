signal on syntax name top
call outer
exit
top:
  do line over condition('O')~traceback
    say '['line']'
  end
  raise propagate
::routine outer
  signal on syntax
  say 1/0
  return
syntax:
  x = s()
  return
s:
  raise propagate
