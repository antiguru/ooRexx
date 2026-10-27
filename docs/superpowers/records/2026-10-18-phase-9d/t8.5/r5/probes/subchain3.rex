/* as subchain.rex, each forwarded or rendered position timed alone */
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
call time 'R'; x = first~string; say 'string' x time('E')
call time 'R'; x = first~request('STRING'); say 'request' x time('E')
call time 'R'; x = first~length; say 'length' x time('E')
call time 'R'; x = first || 'x'; say 'concat' x time('E')
call time 'R'; x = (first = 'leaf'); say 'compare' x time('E')
trace r
x = first
trace o
say 'end'
exit
syntax: say 'SYNTAX' condition('O')~code; exit
::class m subclass stem
