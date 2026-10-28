.local~o = .stream~new('STDOUT')
.o~lineout('kept in local')
.local~f = .stream~new('k.txt')
.f~lineout('file in local')
say 'end'
