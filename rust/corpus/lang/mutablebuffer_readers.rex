/* The readers over a MutableBuffer's contents -- substr, [], pos, lastPos,
   countStr, verify, subWord, word, wordIndex, wordLength, words, wordPos,
   contains, containsWord, startsWith, match, matchChar and subChar -- each
   answer what the oracle answers, and none of them changes the buffer.
   Rendering the buffer itself -- say, concatenation, makeString -- is
   deliberately absent. */
buf = .MutableBuffer~new('abcabc')
/* substr pads past the end; [] defaults its length to one byte, caps it at
   the end and never pads. */
say buf~substr(2) '|' buf~substr(2, 3) '|' buf~substr(5, 4) '|' buf~substr(5, 4, '-') '|' buf~substr(9, 2, '*') '|' buf~substr(9) '|' buf~substr(1, 0)
say buf[2] '|' buf[2, 3] '|' buf[5, 10] '|' buf[7] '|' buf[7, 2] '|' buf[1, 0] '|' buf[6]
/* pos and lastPos with a start and a range; contains is pos as a truth value. */
say buf~pos('bc') buf~pos('bc', 3) buf~pos('bc', 3, 2) buf~pos('bc', 3, 4) buf~pos('x') buf~pos('') buf~pos('abc', 7) buf~pos('c', 6)
say buf~lastPos('bc') buf~lastPos('bc', 4) buf~lastPos('bc', 4, 2) buf~lastPos('bc', 4, 3) buf~lastPos('x') buf~lastPos('abc', 99) buf~lastPos('a', 1)
say buf~contains('ca') buf~contains('ca', 4) buf~contains('bc', 3, 2) buf~contains('bc', 3, 4) buf~contains('x') buf~contains('')
say buf~countStr('bc') buf~countStr('x') buf~countStr('abcabc') .MutableBuffer~new('aaaa')~countStr('aa')
/* verify: an empty reference, both options in either case, a range, and a
   start past the end. */
say buf~verify('abc') buf~verify('ab') buf~verify('ab', 'M') buf~verify('x', 'N', 2) buf~verify('ab', 'N', 2, 1) buf~verify('ab', 'n', 2, 2) buf~verify('c', 'match', 2)
say buf~verify('', 'M') buf~verify('', 'N') buf~verify('', 'N', 3) buf~verify('abc', 'N', 9) buf~verify('', 'N', 9) buf~verify('abc', 'M', 6, 1)
/* startsWith, match and matchChar: the empty string never matches, and a
   start or an offset off the end is 0 rather than a refusal. */
say buf~startsWith('ab') buf~startsWith('abcabc') buf~startsWith('abcabcd') buf~startsWith('b') buf~startsWith('')
say buf~match(1, 'abc') buf~match(4, 'abc') buf~match(2, 'abc') buf~match(4, 'xabc', 2) buf~match(4, 'xabcx', 2, 3) buf~match(4, 'xabcx', 2, 4) buf~match(6, 'c') buf~match(6, 'cd')
say buf~match(7, 'a') buf~match(1, '') buf~match(1, 'abc', 4) buf~match(1, 'abc', 2, 3) buf~match(1, 'abc', 4, 0) buf~match(3, 'zc', 2, 1)
say buf~matchChar(1, 'xa') buf~matchChar(2, 'xa') buf~matchChar(6, 'c') buf~matchChar(7, 'c') buf~matchChar(3, '') buf~matchChar(5, 'abc')
say buf~subChar(1) '|' buf~subChar(3) '|' buf~subChar(6) '|' buf~subChar(7)
/* The word readers, on a buffer with leading, repeated and trailing blanks. */
wbuf = .MutableBuffer~new('  now is  the time  ')
say wbuf~words wbuf~wordIndex(1) wbuf~wordIndex(3) wbuf~wordIndex(4) wbuf~wordIndex(5) wbuf~wordLength(1) wbuf~wordLength(4) wbuf~wordLength(5)
say wbuf~word(1) '|' wbuf~word(3) '|' wbuf~word(4) '|' wbuf~word(5)
say wbuf~subWord(2) '|' wbuf~subWord(2, 2) '|' wbuf~subWord(2, 0) '|' wbuf~subWord(5) '|' wbuf~subWord(4, 5) '|' wbuf~subWord(1, 1)
say wbuf~wordPos('the time') wbuf~wordPos('the') wbuf~wordPos('is', 3) wbuf~wordPos('now', 1) wbuf~wordPos('xx') wbuf~wordPos('') wbuf~wordPos('the   time') wbuf~wordPos('time', 4) wbuf~wordPos('time', 5)
say wbuf~containsWord('is') wbuf~containsWord('is', 3) wbuf~containsWord('the time') wbuf~containsWord('tim') wbuf~containsWord('')
say .MutableBuffer~new('')~words .MutableBuffer~new('   ')~words .MutableBuffer~new('')~word(1) '|' .MutableBuffer~new('')~subWord(1) '|' .MutableBuffer~new('')~wordIndex(1)
/* None of the readers changed either buffer. */
say buf~string buf~length
say wbuf~string'|'wbuf~length
