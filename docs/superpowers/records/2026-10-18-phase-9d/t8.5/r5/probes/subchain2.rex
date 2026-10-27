/* as subchain.rex, with length(first) timed alone */
parse arg n mode
signal on syntax
first = .m~new
t = first
do i = 2 to n
  u = .m~new
  t[] = u
  t = u
end
if mode = 'cyc' then t[] = first
else t[] = 'leaf'
call time 'R'
x = length(first)
e = time('E')
say 'len' x 'in' e
exit
syntax: say 'SYNTAX' condition('O')~code; exit
::class m subclass stem
