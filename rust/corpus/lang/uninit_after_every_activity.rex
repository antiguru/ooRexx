/* Termination runs UNINIT after every activity has ended: an object dropped
   in a started activity that is still running when main ends finalizes
   after that activity's last line. */
m = .w~new~start('run')
say 'main end'
::class k
::method uninit
  say 'uninit of the object the started activity dropped'
::class w
::method run
  d = .k~new
  drop d
  call SysSleep 0.3
  say 'started activity end'
