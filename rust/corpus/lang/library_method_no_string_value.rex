/* An argument with no string value is refused before the extension runs, and
   the refusal is reported against the package that declared the method, with
   no line. */
r = .Re~new('a*b')
say r~does(.object~new)

::requires 're.cls'
