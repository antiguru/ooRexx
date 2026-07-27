g = "global"
call show 1, 2
call exposed
say "after" g
call recurse 3
exit 0

show: procedure
  use arg a, b
  say "show" a b arg()
  return

exposed: procedure expose g
  say "exposed sees" g
  g = "changed"
  return

recurse: procedure
  use arg n
  if n = 0 then return 0
  say "depth" n
  return recurse(n - 1)
