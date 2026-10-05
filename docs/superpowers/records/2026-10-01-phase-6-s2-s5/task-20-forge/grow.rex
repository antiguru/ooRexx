parse arg f
b = .mutablebuffer~new('abcdefghijklmnopqrstuvwxyz')
m = .t~new~start('grow', b, f)
say HOLDBUFFER(b, f, 'ZZ')
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
  call lineout f, 'done'
  call lineout f
  return 'grown'
