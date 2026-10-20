/* Packages whose loaders run but register no command environment: an
   environment named after one of them is unregistered, rc 30 and FAILURE,
   as is HostEmu while the hostemu package is not loaded. */
call on failure name failed
address rxsock 'hello'
say 'rxsock' rc .rs
address RXSOCK 'hello'
say 'RXSOCK' rc .rs
address rxunixsys 'hello'
say 'rxunixsys' rc .rs
address HostEmu 'hello'
say 'HostEmu' rc .rs
exit
failed:
  say 'failure' condition('d') rc
  return
::requires "rxsock" LIBRARY
::requires "rxunixsys" LIBRARY
