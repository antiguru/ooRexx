/* A file's classes install, and run their ACTIVATE methods, in the order
   that takes each time the first class in source order whose SUBCLASS,
   INHERIT and METACLASS targets declared in the file are installed. */
say .local~order

::class a subclass c
::method activate class
  call note 'a'
::class b
::method activate class
  call note 'b'
::class c
::method activate class
  call note 'c'
::class d subclass a inherit e
::method activate class
  call note 'd'
::class e mixinclass object
::method activate class
  call note 'e'
::class f metaclass g
::method activate class
  call note 'f'
::class g subclass class
::method activate class
  call note 'g'

::routine note
  use arg name
  if .local~hasEntry('ORDER') then .local~order = .local~order name
  else .local~order = name
