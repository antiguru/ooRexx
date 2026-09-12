/* A stream write requires a string *value*, not a string rendering: an object
   whose class defines makeString writes what that answers, and an array writes
   its items joined by a newline. An object answering neither refuses 88.909,
   which is not covered here -- the oracle attributes that refusal to the REXX
   package where this crate names the program, and that is a separate defect. */
zz = .stdout~lineout(.M~new)
zz = .stdout~charout(.M~new)
zz = .stdout~charout('|')
zz = .stdout~lineout(.array~of('p','q'))
zz = .stderr~lineout(.M~new)
zz = .stdout~lineout('plain')
say 'done'

::class m
::method makestring
  return 'MADE'
