/* METACLASS: what a metaclass donates, and to whom.

   A metaclass gives its *instance* methods to the class side of every class
   built from it, and that donation is what every line below reads the edge
   through. S is usable as a metaclass because MIXINCLASS Class derives it
   from .Class, where metaclass-ness starts.

   K names S. J names no metaclass and gets S anyway, because a directive
   with no METACLASS derives from its superclass's -- K's. M is a mixin with
   a metaclass, the same construction reached through mixinClass(). Q's
   metaclass is declared below it in the file, so the install order is what
   makes the reference resolve, and S2 answering differently from S is what
   says which one Q got.

   ABSTRACT on K is the neighbouring success for class_abstract_metaclass.rex:
   K is built from a metaclass and is not one, so it takes the keyword. */
say .K~classSideHi
say .J~classSideHi
say .M~classSideHi
say .Q~classSideHi
say .M~baseClass

::CLASS S MIXINCLASS Class
::METHOD classSideHi
  return 'from the metaclass'

::CLASS K METACLASS S ABSTRACT

::CLASS J SUBCLASS K

::CLASS M MIXINCLASS Object METACLASS S

::CLASS Q METACLASS S2

::CLASS S2 MIXINCLASS Class
::METHOD classSideHi
  return 'from the second metaclass'
