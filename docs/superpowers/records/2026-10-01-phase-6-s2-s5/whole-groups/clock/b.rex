call time 'r'
x = 0
do i = 1 to 300000; x = x + 1; end
.o~new~m
::class o
::method m
  y = time('R'); z = time('R'); say 'method fresh:' (y <= z)
