r = .runner~new~start('go')
call SysSleep 0.2
st = SysWait()
say 'wait' st
say 'command rc' r~result
::class runner
::method go
  'exit 3'
  'sleep 0.6; exit 5'
  return rc
