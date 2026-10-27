/* chain A1 -> B1 -> A2 -> B2 ... -> 'leaf'; A has no FOO, B defines it: A~foo forwards one hop; time n sends, one per A */
parse arg n
signal on syntax
as = .array~new(n)
first = .a~new; t = first
do i = 1 to n
  as[i] = t
  b = .b~new; b~id = i
  t[] = b
  if i < n then do; u = .a~new; b[] = u; t = u; end
  else b[] = 'leaf'
end
call time 'R'
s = 0
do i = 1 to n
  s = s + as[i]~foo
end
say 'sum' s 'in' time('E')
exit
syntax: say 'SYNTAX' condition('O')~code condition('O')~position; exit
::class a subclass stem
::class b subclass stem
::attribute id
::method foo
  return self~id
