parse arg f
b = .mutablebuffer~new('abcdefghijklmnopqrstuvwxyz')
m = .t~new~start('overlay', b, f)
say HOLDBUFFER(b, f, 'ZZ')
say b~substr(1, 4) b~length
say m~result
::requires 'lent' LIBRARY
::class t
::method overlay
  use arg b, f
  do 500 until SysFileExists(f'.held')
    call SysSleep 0.01
  end
  b~overlay('XY', 3)
  call lineout f, 'done'
  call lineout f
  return 'overlaid'
