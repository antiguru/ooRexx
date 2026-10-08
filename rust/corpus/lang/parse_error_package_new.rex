/* Package~new over source lines that do not parse raises SYNTAX at the
   failing line, trapped and then untrapped. */
signal on syntax
p = .package~new('x', .array~of('say 1', 'say (  ; nop'))
say 'not reached'
exit
syntax:
  o = condition('O')
  say o~code o~position rc o~traceback~items
  do l over o~traceback; say '  tb:' l; end
  p = .package~new('y', .array~of('nop', 'lab: if then'))
  say 'not reached'
