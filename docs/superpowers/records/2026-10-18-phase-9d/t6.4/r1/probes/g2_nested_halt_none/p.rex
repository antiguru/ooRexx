
call r
if var('RESULT') then say 'R' result; else say 'R none'
say 'end'
exit 0
h:
  o = condition('o')
  say 'H' condition('c') '|' condition('i') '|' condition('d') '|' o~propagated '|' o~hasindex('RESULT') '|' o~result '|' o~instruction
  if condition('i') == 'SIGNAL' then exit 0
  return
::routine r
  signal on halt name ha
  call r2
  say 'r: after r2'
  return 'r-normal'
ha:
  say 'ha' condition('c') condition('i') condition('o')~propagated
  raise propagate
  return 'r-fell'
::routine r2
  signal on halt name hh
  call sub 'halt'
  say 'not reached'
  return 'normal'
hh:
  raise propagate
  say 'after propagate'
  return 'fell'
sub:
  if arg(1) == 'user' then raise user x description 'dd'
  raise halt description 'dd'
