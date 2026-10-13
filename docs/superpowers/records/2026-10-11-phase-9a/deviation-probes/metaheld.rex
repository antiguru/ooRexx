x = .class~new('XX')
.local~held = x
k = .object~subclass('k', .meta)
say k~id k~superclass~id (k == x) x~id x~superclass~id
::class meta subclass class
::method new class
  return .held
