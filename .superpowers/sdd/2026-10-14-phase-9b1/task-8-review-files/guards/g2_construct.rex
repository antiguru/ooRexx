m = .mm~new('a', 'return 1')
say m~class~id
p = .mp~new('pk', .array~of('::routine r public', 'return 2'))
say p~class~id
q = .mq~new('pk2', .array~of('return 3'))
say q~class~id
say .mm~new('b', .array~of('return 4'))~class~id
r = .mr~new('r', 'return 5')
say r~class~id r~call
exit
::class mm subclass method
::class mp subclass package
::class mq subclass package
::class mr subclass routine
