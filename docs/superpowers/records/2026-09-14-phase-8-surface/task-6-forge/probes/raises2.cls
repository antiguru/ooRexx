::options error syntax failure syntax
::routine escalate public
  signal on syntax
  address xd 'RAISE ERROR'
  say 'not here'
  syntax:
  c = condition('o')
  say 'escalated' c~code c~message rc
  signal on syntax name s2
  address xd 'RAISE FAILURE'
  say 'not here'
  s2:
  c = condition('o')
  say 'escalated' c~code c~message rc
  signal on syntax name s3
  address xd 'RAISENORES FAILURE'
  s3:
  c = condition('o')
  say 'escalated' c~code c~message rc
  return
