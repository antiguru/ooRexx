/* .context~thread: B starts C and waits on it, so C ends first and D, started
   after both, takes C's number. */
o = .t~new
main = .context~thread
mb = o~start('b')
parse value mb~result with b c
md = o~start('d')
say 'main' main 'D' md~result 'B' b 'C' c
::class t
::method b unguarded
  n = .context~thread
  mc = self~start('c')
  return n mc~result
::method c unguarded
  return .context~thread
::method d unguarded
  return .context~thread
