/* A CALL ON handler gets the condition object as its one argument, whether
   it is a label, a builtin or a routine. */
call on user u name label_handler
call raise_in 'user u'
call on error name label_handler
'exit 3'
call on notready name label_handler
call charin '/'
call on user v name reverse
signal off nostring
call raise_in 'user v'
say 'reverse ran'
call on user w name routine_handler
call raise_in 'user w'
call off error
call on any name routine_handler
'exit 4'
call on halt name routine_handler
call raise_in 'halt'
say 'done'
exit

label_handler:
  object = condition('O')
  say 'label' arg() arg(1)~class~id arg(1)~condition arg(1)~instruction,
    arg(1)~equivalent(object) (arg(1) == object) condition('S')
  return

::routine raise_in
  interpret 'raise' arg(1) "description 'd-" || word(arg(1), words(arg(1))) || "'"

::routine routine_handler
  use strict arg condition
  say 'routine' arg() condition~condition condition~instruction condition~description
