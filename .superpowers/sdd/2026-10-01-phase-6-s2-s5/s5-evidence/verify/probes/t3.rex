o = .k~new
say 'main end'
exit 7
::class k
::method uninit
  say 'uninit starts'
  .z~new~start('go')
  say 'uninit after start'
::class z
::method go
  call SysSleep 0.2
  say 'z go on' .context~thread
::method bad
  call SysSleep 0.2
  say 'z fails'
  x = 1/0
