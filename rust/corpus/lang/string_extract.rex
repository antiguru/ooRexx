/* String's extraction readers -- substr, [], subChar, subWord and subWords.
   substr pads past the end; [] defaults its length to one byte, caps it at
   the end and never pads; subWords answers an Array rather than a string,
   which is why its size and class are asked for rather than inferred from
   the rendering. */
s = 'abcabc'
say s~substr(2) '|' s~substr(2, 3) '|' s~substr(5, 4) '|' s~substr(5, 4, '-') '|' s~substr(9, 2, '*') '|' s~substr(9) '|' s~substr(1, 0)
say s[2] '|' s[2, 3] '|' s[5, 10] '|' s[7] '|' s[7, 2] '|' s[1, 0] '|' s[6]
say s~subChar(1) '|' s~subChar(3) '|' s~subChar(6) '|' s~subChar(7)
w = '  now is  the time  '
say w~subWord(2) '|' w~subWord(2, 2) '|' w~subWord(2, 0) '|' w~subWord(5) '|' w~subWord(4, 5) '|' w~subWord(1, 1)
say w~subWords(2)~items w~subWords(2, 2)~items w~subWords(5)~items w~subWords(1, 1)~items
say w~subWords(2)~class~id w~subWords(2)[1] w~subWords(2)[3]
say ''~subWord(1) '|' ''~subWords(1)~items '|' ''~substr(1) '|' ''~subChar(1)
say s~substr(2)~class~id s[2]~class~id
say s'|'w'|'
