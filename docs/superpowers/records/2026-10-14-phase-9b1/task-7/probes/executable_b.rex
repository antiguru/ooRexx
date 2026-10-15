n = .object~method('STRING')
.k~define('S', n)
say 1 .k~new~s
.k~defineMethods(.directory~of(('U', .object~method('CLASS'))))
say 3 .k~new~u
say 4 .k~new~run(.object~method('CLASS'))
say 5 n~copy~isGuarded
say 6 .routine~new('x', 'return 5')~call
::class k
