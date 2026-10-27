/* a message forwarded through a reference to a stem whose default is cyclic */
signal on syntax
s. = >s.
q = >s.
say q~items q~hasIndex(1) q~allIndexes~items
q~put('a', 1)
say q~items q~hasIndex(1) q~allIndexes~items s.1
say q~length
say 'end'
exit
syntax: say 'SYNTAX' condition('O')~code; exit
