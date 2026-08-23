/* The accessor pair a `::ATTRIBUTE` and a `::METHOD ... ATTRIBUTE` generate,
   which read and write one variable in a variable pool on the receiver, keyed
   by the class the accessor was declared in.

   The variable is the directive's name AS WRITTEN, where the two message
   names are that name upcased and that name upcased with `=` appended. The
   `bB` pair below is what tells the two apart: an accessor keyed on the
   message name would answer `BB` for the uninitialised read and the pair
   would still round-trip, so the round trip alone cannot see it.

   The pool entry is the one `EXPOSE` reaches, in both directions, which is
   what makes a generated accessor and a written class method two doors onto
   one variable rather than two variables. And it is keyed on the receiver as
   well as on the scope: `.J~e` and `.K~e` are one declaration and two pools,
   so the second reads the derived name `E` after the first was assigned.

   `GET` and `SET` each generate their own half and neither generates the
   other; the halves that are missing here are `corpus/lang/`'s own 97.1
   witnesses, not this program's.

   A value goes in and comes out, rather than a rendering of one: the last
   line stores a class object and reads it back. */

.K~a = 5
say .K~a

.K~b = 'stored'
say .K~b .K~bB

.J~e = 'on J'
say .J~e .K~e

.K~c = 'through the setter'
say .K~readC
x = .K~writeD
say .K~d

say .K~g
.K~h = 'discarded'
say 'the setter has no getter beside it'

.K~a = .K
say .K~a

::class K

::attribute a class
::method b class attribute
::attribute "bB" class
::attribute c class
::attribute d class
::attribute e class
::attribute g class get
::attribute h class set

::method readC class
  expose c
  return c

::method writeD class
  expose d
  d = 'through EXPOSE'
  return 1

::class J subclass K
