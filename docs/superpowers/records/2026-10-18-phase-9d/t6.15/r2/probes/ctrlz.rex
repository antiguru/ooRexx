/* A trailing ctrl-Z: the write position each open reports, and what a
   CHAROUT and a LINEOUT at the end leave. */
call mk; call stream 'z', 'c', 'open both nobuffer'
say 'both w' stream('z', 'c', 'query position write') 'charout' charout('z', 'Q'); call stream 'z', 'c', 'close'; call show
call mk; call stream 'z', 'c', 'open both nobuffer'
say 'both w' stream('z', 'c', 'query position write') 'lineout' lineout('z', 'x'); call stream 'z', 'c', 'close'; call show
call mk; call stream 'z', 'c', 'open both append nobuffer'
say 'append w' stream('z', 'c', 'query position write') 'lineout' lineout('z', 'x'); call stream 'z', 'c', 'close'; call show
call mk; say 'implicit charout' charout('z', 'Q'); call stream 'z', 'c', 'close'; call show
exit
mk: call stream 'z', 'c', 'open write replace'; call charout 'z', 'ab' || '0a'x || '1a'x; call stream 'z', 'c', 'close'; return
show: say '   ' c2x(charin('z', 1, 20)); call stream 'z', 'c', 'close'; return
