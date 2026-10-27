/* does a bare stem's conversion raise NOSTRING on a string default */
signal on nostring
s. = 5
say s.
say 'p' s.
t. = .array~of(1,2)
say t.
say 'end'
exit
nostring: say 'NS' condition('D') condition('A')~class~id; exit
