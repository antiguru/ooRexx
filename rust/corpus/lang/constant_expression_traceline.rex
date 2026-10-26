/* A ::CONSTANT expression's level and the level installing its package
   each echo their own directive: the ::CONSTANT and the ::CLASS, in the
   live frames and in a trapped condition's. */
say .c~k
say .c~t
::class c
::constant k (.c~live)
::constant t (.c~trapped)
::method live class
  do f over .context~stackframes
    say f~type filespec('name', f~name) f~line
    say '   ' f~traceline
  end
  return 1
::method trapped class
  signal on syntax
  x = 1 / 0
syntax:
  do f over condition('o')~stackframes
    say f~type filespec('name', f~name) f~line
    say '   ' f~traceline
  end
  return 2
