call trial 'HASH'
call trial 'EQUAL'
exit
trial: procedure
  use arg mode
  signal on syntax name failed
  t = .table~new
  t~put('held', .key~new('NONE'))
  say t~remove(.key~new(mode))
  say 'unreached'
  return
failed: say 'SYNTAX' condition('o')~code 'items' t~items; return
::class key
::method init
  expose mode
  use arg mode
::method hashCode
  expose mode
  if mode == 'HASH' then raise syntax 98.900 array('hash raised')
  return 'key'
::method '=='
  expose mode
  if mode == 'EQUAL' then raise syntax 98.900 array('equal raised')
  return 1
