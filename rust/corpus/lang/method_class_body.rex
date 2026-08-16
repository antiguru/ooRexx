/* A ::METHOD body entered by a message send: self, super, an argument and a
   returned value, on a class method, which is the one kind of method a
   program can reach before ~new exists. */
say .K~greet('world')
say .K~count
say 'main done'

::class K

::method greet class
  use arg who
  say 'self is' self
  say 'super is' super
  return 'hello' who

::method count class
  return 6 * 7
