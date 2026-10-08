/* Method~new and Routine~new over source that does not parse raise SYNTAX
   with the failing clause first, trapped and then untrapped. */
call try .method
call try .routine
r = .routine~new('r', .array~of('nop', 'if 1 then end'))
say 'not reached'
exit
try: signal on syntax
  m = arg(1)~new('m', .array~of('nop', 'if 1 then say (  ; nop'))
  say 'not reached'
  return
syntax:
  o = condition('O')
  say o~code o~position rc o~traceback~items
  do l over o~traceback; say '  tb:' l; end
  return
