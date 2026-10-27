/* a message forwarded hop by hop through n Stem subclass instances */
parse arg n
signal on syntax
first = .m~new
t = first
do i = 2 to n
  u = .m~new
  t[] = u
  t = u
end
t[] = 'leaf'
call time 'R'; x = first~length; say 'length' x time('E')
exit
syntax: say 'SYNTAX' condition('O')~code; exit
::class m subclass stem
