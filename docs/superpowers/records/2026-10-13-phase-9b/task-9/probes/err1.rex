signal on syntax
x = syssleep('abc')
exit
syntax:
  c = condition('o')
  say c~code c~message
  say 'trace' c~traceback~items
  do l over c~traceback; say l; end
  call sysSleep 'q'
