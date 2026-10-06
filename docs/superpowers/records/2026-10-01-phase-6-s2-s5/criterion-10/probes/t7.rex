o = .k~new
say 'main end'
::class k
::method uninit
  say 'uninit starts a poller'
  .z~new~start('go')
::class z
::method go
  do forever
    call SysSleep 1
  end
