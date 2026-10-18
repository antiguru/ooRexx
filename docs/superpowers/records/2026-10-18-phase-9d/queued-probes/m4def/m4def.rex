o = .c~new
.c~define('FOO', 'return "new"')
say o~instanceMethod('FOO')~source[1]
say o~foo
::class c
::method foo
  return 'old'
