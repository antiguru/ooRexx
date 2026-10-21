/* A SIGNAL out of an INTERPRET runs the UNINITs a collection readied, as
 * the fragment's return does: the counter read in the fragment and the one
 * read at the label differ once a collection ran inside the fragment. The
 * attribute reads are not native sends, so nothing else drains between them. */
c = .counter~new
.local~c = c
hits = 0
i = 0
top:
if i > 0, c~n > b then hits = hits + 1
i = i + 1
o = .u~new
o = .nil
if i < 3000 then interpret 'z = copies("x", 20000) || i; b = c~n; signal top'
say 'drained at the signal' (hits > 0)
::class counter
::attribute n
::method init; expose n; n = 0
::method bump; expose n; n = n + 1
::class u
::method uninit
  .local~c~bump
