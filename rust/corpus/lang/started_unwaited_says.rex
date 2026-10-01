/* A started method nothing waits on still runs: the program's end waits for
   every activity. Main writes nothing, so the order is the oracle's own. */
.k~new~start('HELLO')
::class k
::method hello
  say 'started'
