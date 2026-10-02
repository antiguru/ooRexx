/* The rest of a replied method starts a send and waits on its result: the
   wait parks the continuation's own activity. Main writes nothing, so the
   body's line is the only output. */
x = .w~new~r
::class w
::method r
  reply 'replied'
  m = self~start('s')
  say 'after reply' m~result
::method s unguarded
  return 'S'
