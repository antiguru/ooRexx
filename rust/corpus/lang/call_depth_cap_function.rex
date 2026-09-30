/* Unbounded recursion by function call, trapped in the outermost caller. */
signal on syntax name caught
say 'start'
n = rec(1)
say 'unreached' n
exit
caught:
  say 'caught' rc condition('C')
  say condition('D')
  exit 0
::routine rec
  return rec(arg(1) + 1) + 1
