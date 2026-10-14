/* Do not run on the oracle: SIGSEGV, rc 139, 1 run of 1, 2026-10-14. */
s.0 = 2; s.1 = .k~new; s.2 = 'a'
call SysStemSort 's.'
say 'not reached'
::class k
::method string
  raise syntax 93.900 array('boom')
