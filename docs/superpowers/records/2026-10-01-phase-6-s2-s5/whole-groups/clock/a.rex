o = .o~new
o~m1
o~m2
::class o
::method m1
  call time 'r'
  x = 0
  do i = 1 to 300000; x = x + 1; end
::method m2
  y = time('R'); z = time('R'); say 'm2 fresh:' (y <= z)
