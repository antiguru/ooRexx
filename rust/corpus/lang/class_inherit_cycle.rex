/* An INHERIT target is a dependency exactly as a SUBCLASS target is, so a
   class inheriting itself cannot be ordered: 98.911, naming the program. */
say 'main ran'

::CLASS K INHERIT K
