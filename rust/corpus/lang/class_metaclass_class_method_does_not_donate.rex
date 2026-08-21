/* A metaclass donates its instance methods and not its class methods:
   ::METHOD ... CLASS on the metaclass lands in the metaclass's own class
   dictionary, which nothing merges into K's class behaviour. 97.1 rc 159,
   where the same directive without CLASS answers -- class_metaclass.rex's
   first line. */
say .K~classSideHi

::CLASS S MIXINCLASS Class
::METHOD classSideHi CLASS
  return 'class-side hi'

::CLASS K METACLASS S
