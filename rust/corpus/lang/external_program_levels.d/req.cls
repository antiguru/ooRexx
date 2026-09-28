::class req public
::method go
  use arg z
  interpret 'q = z / 0'
