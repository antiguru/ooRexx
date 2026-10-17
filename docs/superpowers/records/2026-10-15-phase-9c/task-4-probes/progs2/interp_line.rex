nop
nop
call try 'y = (1'
call try 'nop; nop; y = (1'
call try 'if 1 = 1'
call try 'do' || '0a'x || 'nop'
call try 'nop' || '0a'x || 'y = (1'
exit

try:
  signal on syntax name trap
  interpret arg(1)
  say 'ok'
  return
trap:
  c = condition('o')
  say c~code c~position '|' c~message
  return
