/* An arithmetic or prefix operator on a String subclass instance is sent to
   the instance before any numeric conversion, as every other operator is, so
   its own method answers even though '3' is a number. A subclass with no
   operator method of its own answers through String's. A controlled DO sends
   the setup's prefix + to a subclass instance in any of its numeric
   positions. A stem whose default value is one forwards the operator to it. */
s = .mystr~new('3')
say (s + 1) (s - 1) (s * 2) (s / 2) (s % 2) (s // 2) (s ** 2)
say (-s) (+s) (\s) (s < 4) (s || 'x')
t. = s; say (t. + 1) (-t.)
d = .plainsub~new('3'); say (d + 1) (-d) (d * 2) (d ** 2) (\.plainsub~new('0'))
do i = .counter~new('1') to 3; say 'i' i; end
do i = 1 to .counter~new('2') by .counter~new('1'); say 'j' i; end
do i = s to 4; say 'k' i; end
say 'done'
exit
::class mystr subclass string
::method '+'; if arg() = 0 then return 'pfx+'; return 'plus'
::method '-'; if arg() = 0 then return 'pfx-'; return 'minus'
::method '*'; return 'times'
::method '/'; return 'div'
::method '%'; return 'idiv'
::method '//'; return 'rem'
::method '**'; return 'pow'
::method '\'; return 'not'
::method '<'; return 'lt'
::class plainsub subclass string
::class counter subclass string
::method '+'
  say 'sent +' arg()
  forward class (super)
