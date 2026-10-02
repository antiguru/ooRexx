/* A REPLY that is the last clause its method runs, at its top level, ending
   an IF's arm and inside a DO: the sender gets the value. */
o = .k~new
say o~last
say o~lastInIf(1)
say o~lastInSelect
say o~lastInDo
::class k
::method last
  reply 'last'
::method lastInIf
  use arg c
  if c then reply 'then, last'
::method lastInSelect
  select
    when 1 then reply 'when, last'
  end
::method lastInDo
  do 1
    reply 'in a do, last'
  end
