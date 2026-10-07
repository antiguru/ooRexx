/* DELEGATE to a stem name sends to the object's stem, which forwards to its
   default value; a stem with none is made empty on first use. */
o = .t~new
say o~length
say .u~new~items
::class t
::method init
expose a.
a. = 'xyz'
::method length delegate a.
::class u
::method items delegate s.
