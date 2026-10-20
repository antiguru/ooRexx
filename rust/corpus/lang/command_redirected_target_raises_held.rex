/* An output target that raises while its command runs: the clause waits
   for the child, later lines are still offered to the target, the error
   target is written, and the condition the target raised last is raised
   at the command's clause, with no RC or .RS set and no part line
   flushed. */
b = .Bad~new
e.0 = 9
signal on syntax name raised
address system 'echo a; echo e1 >&2; sleep 0.4; echo b; echo c; printf part' -
  with output using (b) error stem e.
say 'not here'
raised:
say 'trapped' condition('o')~code 'calls' b~calls 'lines' b~lines
say 'rc' rc '.rs' .rs 'error' e.0 e.1
exit

::class Bad subclass OutputStream
::attribute calls
::attribute lines
::method init
  self~calls = 0
  self~lines = ''
::method lineout
  use arg line
  self~calls += 1
  if self~calls = 1 then return 1 / 0
  if line == 'c' then return 'x' + 1
  self~lines = self~lines line
  return 0
