/* A replied body, run at the program's end, starts a send and waits on its
   result: the wait runs the started activity from the body's own frames.
   Main writes nothing, so the body's line is the only output. */
x = .w~new~r
::class w
::method r
  reply 'replied'
  m = self~start('s')
  say 'after reply' m~result
::method s unguarded
  return 'S'
