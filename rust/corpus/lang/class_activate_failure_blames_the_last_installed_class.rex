/* A failure inside a class-side ACTIVATE echoes the class installed LAST,
   not the class whose ACTIVATE raised and not the last ::CLASS in the file.

   ACTIVATE is a walk of its own after every class is built, so the clause
   the report blames alongside it is whatever the install pass reached last
   -- which here is neither A, whose method raised, nor the file's last
   ::CLASS directive: the chain installs parent, then middle, then leaf, and
   leaf is the FIRST class directive in the file. */
say "main"

::CLASS A
::METHOD activate CLASS
  x = 1/0

::CLASS leaf SUBCLASS middle

::CLASS parent

::CLASS middle SUBCLASS parent
