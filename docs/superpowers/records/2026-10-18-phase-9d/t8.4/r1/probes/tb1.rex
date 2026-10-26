say .c~k
::class c
::constant k (.c~f)
::method f class
  signal on syntax
  x = 1 / 0
syntax:
  do f over condition('o')~stackframes; say f~type f~name f~line; say '   ' f~traceline; end
  return 1
