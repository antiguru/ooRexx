/* ObjectToStringValue of a non-string kept by a global reference on the object, read in the next call, no collection */
o = .array~of(1,2)
call KeepOTS o
say c2x(ReadOTS())
o = .array~of('a much longer element that is not inline', 'x')
call KeepOTS o
say ReadOTS()
::requires 'rr' LIBRARY
