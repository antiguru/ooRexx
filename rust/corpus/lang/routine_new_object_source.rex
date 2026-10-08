/* Routine~new with a source that is neither a string nor an array: 93.961. */
signal on syntax
m = .routine~new('r', .object~new)
say 'no'
exit
syntax: say rc condition('O')~code
