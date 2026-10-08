/* An external routine whose file does not parse raises SYNTAX in that file,
   trapped and then untrapped. */
signal on syntax
call 'badcall.cls'
say 'not reached'
exit
syntax:
  o = condition('O')
  say o~code o~position rc o~traceback~items
  do l over o~traceback; say '  tb:' l; end
  call 'badcall2.cls'
  say 'not reached'
