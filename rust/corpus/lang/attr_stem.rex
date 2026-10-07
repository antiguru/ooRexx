/* ::ATTRIBUTE on a stem name answers the object's stem, made on first use;
   the setter takes a stem as it is and wraps anything else as a new stem's
   default. */
o = .t~new
say o~s.~class~id (o~s. == o~s.) o~s.~items
o~s.~put('one', 1)
say o~s.[1] o~peek
o~s. = 'dflt'
say o~s.~class~id o~s.[7] o~s.~items o~peek
t.2 = 'two'
o~s. = t.
say o~s.[2] (o~s. == t.) o~peek
::class t
::attribute s.
::method peek
expose s.
return s.1 s.2
