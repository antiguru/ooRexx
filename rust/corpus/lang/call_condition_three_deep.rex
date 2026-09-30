/* A condition raised three calls deep, trapped by the outermost caller. */
signal on syntax name caught
call on user done name handled
x = 10
y = x + one(1) * 2
say 'y' y
call trio 'u'
say 'after trio' result
select
  when y > 100 then say 'big'
  when two(y) > 0 then say 'two' two(y)
  otherwise nop
end
z = 'keep' || one(0)
say 'unreached' z
exit
caught:
  say 'caught' rc condition('C') sigl
  say condition('D')
  exit 0
handled:
  say 'handled' condition('C') condition('D') sigl
  return
one: procedure
  arg n
  return two(n) + 1
two: procedure
  arg n
  return three(n) * 1
three:
  arg n
  if n = 0 then return 1 / n
  return n
trio:
  call deep 3
  return 'trio' arg(1)
deep:
  if arg(1) > 1 then call deep arg(1) - 1
  else raise user done description 'from deep' return
  say 'deep' arg(1)
  return
