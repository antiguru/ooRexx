call time 'r'
x = 0
do i = 1 to 300000; x = x + 1; end
call r
exit
::routine r
t = time('r')
say 'routine r zero:' (t = 0)
