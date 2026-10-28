say .plain
say .plain~defaultName
say .array
say .withother
say .meta
say .mk
c = .class~new('dyn')
say c
c2 = .withname~subclass('dynsub')
say c2
say .withother
::class plain
::class withother
::method foo class
  return 1
::class meta subclass class
::method defaultname
  return 'METADN'
::class mk metaclass meta
