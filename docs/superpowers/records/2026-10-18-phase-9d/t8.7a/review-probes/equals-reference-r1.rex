k = .k~new
say 'kmake'~equals(>k) 'a K'~equals(>k) 'KMAKE'~caselessEquals(>k)
say 'kmake'~equals(k)
y = 'abc'
say 'abc'~equals(>y) 'Y'~equals(>y)
say 'abc'~left(1) 'abcdef'~left(>y~length)
call t "'abc'~pos(>y)"
call t "'abc'~compareTo(>y)"
signal on nostring name ns
say 'kmake'~equals(>k)
exit
ns: say 'NOSTRING' condition('d') condition('o')~class
exit
t:
  signal on syntax
  interpret 'r = ' arg(1)
  say arg(1) '->' r
  return
syntax:
  say arg(1) '->' condition('o')~code condition('o')~message
  return
::class k
::method makestring
  return 'kmake'
