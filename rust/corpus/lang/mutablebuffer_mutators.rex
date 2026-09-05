/* The mutators over a MutableBuffer's contents -- insert, overlay,
   replaceAt, []=, changeStr, upper, lower, translate, space, delWord and
   delete -- each change the buffer in place and answer the receiver, and
   the ones that can outgrow the capacity raise it. Rendering the buffer
   itself -- say, concatenation, makeString -- is deliberately absent. */
buf = .MutableBuffer~new('abcdef')
/* insert: the position is 0-based and defaults to 0; a position past the
   end pads the gap and a length longer than the string pads the insertion,
   with a blank where no pad is given; a zero-length insertion inside the
   contents changes nothing. */
say buf~insert('XY')~string buf~length
say buf~insert('Z', 3)~string
say buf~insert('', 2)~string buf~insert('QR', 2, 0)~string
say buf~insert('QR', 20, 4, '-')~string buf~length
say buf~insert('PQ', 2, 5, '.')~string buf~insert('W', 2, 1)~string
say .MutableBuffer~new('abcdef')~insert('QR', 9)~string'|' .MutableBuffer~new('abcdef')~insert('QR', 2, 5)~string'|'
/* overlay: the position is 1-based and defaults to 1; past the end it pads
   the gap, with a blank where no pad is given; a zero length and an empty
   string with the default length each change nothing. */
buf = .MutableBuffer~new('abcdef')
say buf~overlay('XY')~string
say buf~overlay('Z', 4)~string
say buf~overlay('QR', 9, 4, '-')~string buf~length
say buf~overlay('W', 2, 0)~string buf~overlay('', 3)~string buf~length
say buf~overlay('', 3, 2, '+')~string buf~length
say .MutableBuffer~new('abcdef')~overlay('QR', 10)~string'|' .MutableBuffer~new('abcdef')~overlay('QR', 2, 5)~string'|'
/* replaceAt: the length defaults to the replacement's own, is cut at the
   end of the contents, an empty replacement excises the range, and a
   position past the end pads the gap with a blank where no pad is given. */
buf = .MutableBuffer~new('abcdef')
say buf~replaceAt('XY', 3)~string buf~length
say buf~replaceAt('Z', 3, 2)~string buf~length
say buf~replaceAt('QRST', 2, 1)~string
say buf~replaceAt('', 2, 4)~string buf~length
say buf~replaceAt('P', 9, 2, '-')~string buf~length
say buf~replaceAt('N', 5, 99)~string buf~replaceAt('', 2, 0)~string buf~length
say .MutableBuffer~new('abcdef')~replaceAt('QR', 10)~string'|'
/* []= is replaceAt with the default pad -- it takes no pad argument of its
   own -- under the message name and under the subscript form. */
buf = .MutableBuffer~new('abcdef')
say buf~'[]='('XY', 3)~string
buf[3] = 'Z'
say buf~string buf~length
buf[3, 4] = 'PQ'
say buf~string buf~length
say buf~'[]='('', 2, 2)~string buf~length
say .MutableBuffer~new('abcdef')~'[]='('QR', 10)~string'|'
/* changeStr: equal, shorter and longer replacements, a count that stops
   early, and the three that change nothing -- an absent needle, an empty
   needle and a zero count. */
buf = .MutableBuffer~new('abcabcabc')
say buf~changeStr('bc', 'ZZ')~string buf~length
say buf~changeStr('ZZ', 'Q')~string buf~length
say buf~changeStr('Q', 'RST')~string buf~length
say buf~changeStr('RST', 'Q', 2)~string
say buf~changeStr('zz', 'Q')~string buf~changeStr('', 'Q')~string buf~changeStr('a', 'Q', 0)~string
/* upper and lower: the whole contents, a range, and the two non-changes --
   a start past the end and a zero length. */
buf = .MutableBuffer~new('abcDEF')
say buf~upper~string buf~length buf~getBufferSize
say buf~lower(2, 3)~string
say buf~upper(9)~string buf~lower(2, 0)~string
say buf~lower(2, 99)~string
say .MutableBuffer~new('a1B!c')~upper~string
/* translate: an output table with an input table, a shorter output table
   padding, an output table alone, an input table alone, a range, and the
   no-table form, which is upper. */
buf = .MutableBuffer~new('abcdef')
say buf~translate('ABC', 'abc')~string
say buf~translate('xy', 'ABC', '-')~string
say buf~translate('QQ', 'def', , 4, 2)~string
say buf~translate('A', 'a', , 9)~string buf~translate('A', 'a', , 1, 0)~string
say .MutableBuffer~new('abcdef')~translate('ABC')~string'|'
say .MutableBuffer~new('abcdef')~translate(, 'abc')~string'|'
say .MutableBuffer~new('abcdef')~translate(, , '?')~string
say .MutableBuffer~new('abcdef')~translate~string
say .MutableBuffer~new('abcdef')~translate(, , , 2, 3)~string
/* space: the default single blank, none, several, a pad, and the two that
   have no interstice to fill -- one word and no words. */
say .MutableBuffer~new('  now is  the time  ')~space~string'|'
say .MutableBuffer~new('  now is  the time  ')~space(0)~string
say .MutableBuffer~new('  now is  the time  ')~space(3)~string'|'
say .MutableBuffer~new('  now is  the time  ')~space(2, '-')~string
say .MutableBuffer~new('one')~space(4, '=')~string'|'
say .MutableBuffer~new('   ')~space~length .MutableBuffer~new('')~space(2)~length
/* delWord: from a word to the end, a count, and the three non-changes -- a
   position past the last word, a zero count and a buffer with no words. */
say .MutableBuffer~new('  now is  the time  ')~delWord(2)~string'|'
say .MutableBuffer~new('  now is  the time  ')~delWord(2, 1)~string'|'
say .MutableBuffer~new('  now is  the time  ')~delWord(1, 2)~string'|'
say .MutableBuffer~new('  now is  the time  ')~delWord(5)~string'|'
say .MutableBuffer~new('  now is  the time  ')~delWord(2, 0)~string'|'
say .MutableBuffer~new('   ')~delWord(1)~length .MutableBuffer~new('')~delWord(1)~length
/* delete is delStr's method under its second name: a start, a start and a
   length, an omitted start, and a start past the end. */
say .MutableBuffer~new('abcdef')~delete(2)~string'|'
say .MutableBuffer~new('abcdef')~delete(2, 3)~string
say .MutableBuffer~new('abcdef')~delete(, 2)~string
say .MutableBuffer~new('abcdef')~delete(99)~string .MutableBuffer~new('abcdef')~delete(2, 0)~string
say .MutableBuffer~new('abcdef')~delete~length
/* Every mutator answers the receiver itself, and they chain. */
buf = .MutableBuffer~new('  now is  the time  ')
say (buf~upper == buf) (buf~space == buf) (buf~insert('') == buf) (buf~delete(99) == buf)
say (buf~overlay('n', 1) == buf) (buf~replaceAt('N', 1) == buf) (buf~changeStr('z', 'Q') == buf)
say (buf~lower == buf) (buf~translate('A', 'a') == buf) (buf~delWord(9) == buf)
say .MutableBuffer~new('  now is  the time  ')~upper~space~delWord(2, 1)~string
/* Growth: insert, overlay, replaceAt, []=, changeStr and space each raise
   the capacity past the size the buffer was built with, and past the 256
   the constructor defaults to. */
grow = .MutableBuffer~new('abc', 10)
say grow~insert(copies('y', 40), 0)~length grow~getBufferSize
say grow~insert('Q', 400)~length grow~getBufferSize
grow = .MutableBuffer~new('abc')~insert(copies('y', 400), 0)
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('abc')~overlay('Z', 400)
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('abc', 10)~overlay('Z', 400)
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('abc')~replaceAt(copies('z', 400), 2)
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('ab')~'[]='(copies('w', 300), 1)
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('abcabc')~changeStr('b', copies('q', 200))
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('a b c', 10)~space(20)
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('a b c')~space(200)
say grow~length grow~getBufferSize
