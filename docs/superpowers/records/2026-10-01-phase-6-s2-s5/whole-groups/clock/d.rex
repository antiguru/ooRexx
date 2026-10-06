.o~new~m1
x = 0
do i = 1 to 300000; x = x + 1; end
y = time('E'); say 'caller untouched:' (y = 0)
::class o
::method m1
  call time 'r'
