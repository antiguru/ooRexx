/* A target line that raises ends its piece: the piece's part line is
   dropped rather than joined to the next piece's first line. */
o = .Bad~new
signal on syntax name raised
address system 'printf "a\nb\nc"; sleep 0.4; printf "d\ne\n"' with output using (o)
say 'not here'
raised:
say 'trapped' condition('o')~code 'calls' o~calls
exit

::class Bad subclass OutputStream
::attribute calls
::method init
  self~calls = 0
::method lineout
  use arg line
  self~calls += 1
  say 'O:'line
  if line == 'a' then return 1 / 0
  return 0
