/* Message~halt asks the activity the message was last sent from: a message
   never sent answers 0, and one sent from main halts main. */
signal on halt name h
say .message~new(.k~new, 'm')~halt
m = .message~new(.k~new, 'm')
say m~send
say m~halt('stop')
say 'not reached'
exit
h:
say 'halted' condition('D') sigl
say 'again' m~halt
say 'not reached'
::class k
::method m
  return 'v'
