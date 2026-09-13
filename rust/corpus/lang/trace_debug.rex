/* Interactive debug: the banner, the prompt, and what a line typed at one
   does.

   The banner names the `PARSE SOURCE` string and is printed once, ahead of
   the first clause the setting traces; the prompt is printed once per
   activation, at the first pause. An empty line continues, `=` runs the same
   clause again so it traces twice, and anything else runs as a fragment that
   traces nothing of its own.

   A `TRACE` instruction is ignored while debug is on, so `TRACE()` still
   answers the `?` form; the same instruction typed at the prompt is not
   ignored, and `trace off` there ends both the session and the pause. */

say 'A'
trace ?r
zz = 1
yy = zz + 1
say 'B'
say 'end' trace()
