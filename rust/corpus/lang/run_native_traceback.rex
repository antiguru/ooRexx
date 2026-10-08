/* A primitive run runs is blamed as *UNNAMED* with scope .NIL. */
say .t~new~go
::class t
::method go
  return self~run(.object~method('HASMETHOD'))
