/* Task 23 fix round 3: the lock on a library class refuses *and leaves the
   class alone*, which the `_inherit` and `_define` rows beside it cannot
   see: both halt on the refusal, so neither reads the state afterwards. The
   SYNTAX handler is what gets the read to run. Without the flag the send
   succeeds, `.Alarm` gains `Comparable`, and this prints `no refusal`. */
signal on syntax
.Alarm~inherit(.Comparable)
say 'no refusal'
exit
syntax:
say .Alarm~isA(.Comparable)
