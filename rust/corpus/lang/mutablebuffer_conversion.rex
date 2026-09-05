/* MutableBuffer's conversions: makeString, makeArray, subWords and setText. */
lf = '0a'x
cr = '0d'x

buf = .MutableBuffer~new('a b  c')
say 'say' buf
say 'ms' buf~makeString buf~string (buf~makeString == buf~string) buf~makeString~class
say 'cmp' (buf == 'a b  c') ('a b  c' == buf) (buf = 'a b  c') ('a b  c' = buf)
say 'ncmp' (buf \== 'a b  c') ('a b  c' \== buf)
say 'len' length(buf) buf~length
say 'cat' ('x' || buf) (buf || 'y')

mt = .MutableBuffer~new('')
say 'mt [' || mt || '] [' || mt~makeString || ']' mt~makeString~length
say 'mtcmp' (mt == '') ('' == mt) (mt = '') ('' = mt)
say 'mtarr' mt~makeArray~items mt~makeArray~dimension mt~makeArray('')~items
say 'mtsw' mt~subWords~items mt~subWords~dimension

one = buf~makeArray
say 'ma1' one~items one~size one~dimension '[' || one[1] || ']'
lines = .MutableBuffer~new('one' || lf || 'two' || lf)
two = lines~makeArray
say 'ma2' two~items '[' || two[1] || '][' || two[2] || ']'
crlines = .MutableBuffer~new('p' || cr || lf || 'q')
three = crlines~makeArray
say 'ma3' three~items three[1]~length three[2]~length
four = crlines~makeArray(lf)
say 'ma4' four~items four[1]~length four[2]~length
sepb = buf~makeArray('b')
say 'ma5' sepb~items '[' || sepb[1] || '][' || sepb[2] || ']'
each = .MutableBuffer~new('abcabc')~makeArray('')
say 'ma6' each~items '[' || each[1] || '][' || each[6] || ']'
whole = buf~makeArray('zz')
say 'ma7' whole~items '[' || whole[1] || ']'
num = .MutableBuffer~new('a2b2c')~makeArray(2)
say 'ma8' num~items '[' || num[1] || '][' || num[3] || ']'
tail = .MutableBuffer~new('abcabc')~makeArray('c')
say 'ma9' tail~items '[' || tail[1] || '][' || tail[2] || ']'
wide = .MutableBuffer~new('ab')~makeArray('abcd')
say 'ma10' wide~items '[' || wide[1] || ']'

say 'sw0' buf~subWords~class buf~subWords~dimension
all = buf~subWords
say 'sw1' all~items all~size '[' || all[1] || '][' || all[2] || '][' || all[3] || ']'
say 'sw2 [' || buf~subWords(2) || ']'
say 'sw3 [' || buf~subWords(2,1) || ']'
say 'sw4' buf~subWords(2,2)~items buf~subWords(3)~items buf~subWords(4)~items
say 'sw5' buf~subWords(1,0)~items buf~subWords(2,99)~items
say 'sw6 [' || buf~subWords(,2) || ']'

txt = .MutableBuffer~new('abc', 10)
say 'st1' txt~setText('xyz')~string txt~length txt~getBufferSize
say 'st2' txt~setText('')~length txt~getBufferSize
say 'st3' txt~setText(12)~string txt~length
say 'st4' (txt~setText('same') == txt) txt~string
grow = .MutableBuffer~new('abc', 10)
say 'st5' grow~setText(copies('y',40))~length grow~getBufferSize
grow2 = .MutableBuffer~new('abc')
say 'st6' grow2~setText(copies('z',400))~length grow2~getBufferSize
say 'st7' grow2~setText('')~length grow2~getBufferSize
