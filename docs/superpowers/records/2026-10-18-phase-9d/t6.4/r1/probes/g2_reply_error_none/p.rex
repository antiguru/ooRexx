
res = .t~new~m
say 'R' res
call SysSleep 0.5
if var('RESULT') then say 'R' result; else say 'R none'
say 'end'
exit 0
h:
  o = condition('o')
  say 'H' condition('c') '|' condition('i') '|' condition('d') '|' o~propagated '|' o~hasindex('RESULT') '|' o~result '|' o~instruction
  if condition('i') == 'SIGNAL' then exit 0
  return
::class t
::method m
  reply 'replied'
  signal on error name hh
  'false'
  say 'not reached'
  return 'normal'
hh:
  raise propagate
  say 'after propagate'
  return 'fell'
sub:
  if arg(1) == 'user' then raise user x description 'dd'
  raise halt description 'dd'
