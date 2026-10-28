call r
say 'y'
exit
r: procedure
.environment~setentry('KEEPER', .stream~new('STDOUT'))
.keeper~lineout('env kept')
return
