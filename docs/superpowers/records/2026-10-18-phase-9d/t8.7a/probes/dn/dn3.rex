say .mk~defaultName
say .mk
say .withother~defaultName
say .withother
e = .plain~enhanced(.table~new~~put("return ENH", "DEFAULTNAME"))
say e
::class plain
::class withother
::method foo class
  return 1
::class meta subclass class
::method defaultname
  return 'METADN'
::class mk metaclass meta
