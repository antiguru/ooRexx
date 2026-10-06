call time 'r'
x = 0
do i = 1 to 300000; x = x + 1; end
m = .o~new~start('m')
m~wait
::class o
::method m
  y = time('R'); z = time('R'); say 'started fresh:' (y <= z)
