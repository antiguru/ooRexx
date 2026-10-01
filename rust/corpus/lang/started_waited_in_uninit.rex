/* An UNINIT run at termination starts a send and waits on its result. */
u = .u~new
say 'main end'
::class u
::method uninit
  m = .v~new~start('s')
  say 'uninit got' m~result
::class v
::method s
  return 'S'
