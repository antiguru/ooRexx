f = .stream~new('zz_wo.txt')
say 'open' f~open('write replace')
say 'lineout' f~lineout('f1') f~lineout('f2')
say 'lines' f~lines f~lines('C') f~lines('N')
say 'chars' f~chars
say 'bif' lines('zz_wo.txt') lines('zz_wo.txt', 'C') chars('zz_wo.txt')
say 'linein' f~linein f~state f~description
say 'lines2' f~lines
say 'close' f~close
g = .stream~new('zz_wo.txt')
say 'open' g~open('write append')
say 'lines' g~lines g~lines('C') g~chars
say g~close
