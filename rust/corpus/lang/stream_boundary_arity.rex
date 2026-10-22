/* An argument too many for a Stream method is refused at the REXX package's
 * native boundary: the report names REXX, with no line. */
say 'start'
x = .stdout~description(1)
say 'not reached'
