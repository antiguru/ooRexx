m = .t~new~start('ticks')
address system 'sleep 1' with output stem o.
say 'main' rc
::class t
::method ticks
  do i = 1 to 3
    say 'w' i
    call SysSleep 0.1
  end
