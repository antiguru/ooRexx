/* Metaclass-ness travels down SUBCLASS as well as MIXINCLASS: T derives from
   S, which derives from .Class, so T is a metaclass too and ABSTRACT on it
   is the same 98.990. */
say 'main ran'

::CLASS S MIXINCLASS Class

::CLASS T SUBCLASS S ABSTRACT
