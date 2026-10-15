call on user hit name handler
call mk
call filler
say 'q14' length(.local['C']~additional[1]~value) left(.local['C']~additional[1]~value, 3)
exit
handler:
  .local['C'] = condition('o')
  return
mk: procedure
  y = copies('cd', 12)
  raise user hit additional (>y)
  return
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
