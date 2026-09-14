/* A raise the native boundary makes for itself is reported against the file
   the EXTERNAL directive was written in, with no line: the missing argument
   is refused before any of the extension's own code runs. */
r = .Re~new('a*b')
say r~doparse()

::requires 're.cls'
