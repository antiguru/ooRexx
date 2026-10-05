parse arg f
b = .mutablebuffer~new('abcdefghijklmnopqrstuvwxyz')
m = .t~new~start('grow', b, f)
say left(HOLDBUFFER(b, f, 'ZZ'), 4)~c2x
say b~substr(1, 4) b~length
say m~result
::requires 'lent' LIBRARY
::class t
::method grow
  use arg b, f
  do 500 until SysFileExists(f'.held')
    call SysSleep 0.01
  end
  b~append(copies('y', 5000))
  do i = 1 to 300000
    a = .array~new(10)
    s = copies('q', 30)
  end
  call lineout f, 'done'
  call lineout f
  return 'grown'
