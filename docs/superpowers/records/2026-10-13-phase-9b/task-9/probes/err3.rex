signal on syntax
call 'SysSleep' 'abc'
exit
syntax:
  c = condition('o')
  say c~code c~message
  do l over c~traceback; say l; end
