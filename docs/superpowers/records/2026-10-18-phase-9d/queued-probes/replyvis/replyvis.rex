o = .k~new
say o~m
say 'after' o~get
::class k
::method m
  expose v
  v = 'none'
  reply 'replied'
  call SysSleep 0.2
  v = 'argument-value-long'
::method get
  expose v
  return v
