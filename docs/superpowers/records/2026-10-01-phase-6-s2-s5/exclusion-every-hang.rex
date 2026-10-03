/* MutexSemaphore.testGroup TEST_EXCLUSION outside ooTest, with the worker
   yielding after step = 6: main takes the mutex first and the oracle hangs
   (3 of 3, rc 137 under timeout -k 5 20). Ruling P46. */
o = .t~new
o~test_exclusion
::class t
::method test_exclusion
  expose step
  sem = .MutexSemaphore~new
  step = 1
  self~run_test_exclusion(sem)
  guard off when step == 2
  say 'm1' sem~acquire(0)
  say 'm2' sem~acquire(0.05)
  step = 3
  say 'm3' sem~acquire(1)
  step = 4
  guard off when step = 5
  say 'm4' sem~release
  guard off when step == 6
  say 'm5' sem~acquire(1)
::method run_test_exclusion unguarded
  expose step
  use strict arg sem
  reply
  say 'w1' sem~acquire
  step = 2
  guard off when step = 3
  say 'w2' sem~release
  say 'w3' sem~release
  guard off when step = 4
  say 'w4' sem~acquire(0)
  say 'w5' sem~acquire(0.05)
  step = 5
  say 'w6' sem~acquire(-1)
  say 'w7' sem~release
  step = 6
  call SysSleep 0.01
  say 'w8' sem~acquire
