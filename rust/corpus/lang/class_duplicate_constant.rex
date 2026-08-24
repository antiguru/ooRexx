/* Two ::CONSTANT directives of one name in one class.

   Its own code, distinct from ::METHOD's and ::ATTRIBUTE's, and the check
   that reports it runs before the one that refuses a class-less expression:
   the second directive here would be 99.906 in a file with no ::CLASS. */
say 'prolog'

::CLASS A
::CONSTANT c 5
::CONSTANT c 6
