/* Which ::CLASS a failing ::CONSTANT expression is blamed against, when the
   file's classes do not install in source order. The chain here installs
   parent, then middle, then leaf -- so the blame is the leaf, which is the
   FIRST class directive in the file and the last one installed. */
say 'main ran'

::class leaf subclass middle

::class parent

::constant x (1/0)

::class middle subclass parent
