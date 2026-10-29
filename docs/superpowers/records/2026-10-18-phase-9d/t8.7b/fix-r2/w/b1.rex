call try 'do 2; nop; end; x = f()'
exit
try:
  signal on syntax
  interpret arg(1)
  return
syntax:
  do line over condition('O')~traceback
    say '['line']'
  end
  return
f: procedure
  signal off syntax
  return 1 / 0
