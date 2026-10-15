d = .d~new
a = .a~new
s = .s~new('abc')
.d~define('FOO', 'return "d-new"')
.a~define('FOO', 'return "a-new"')
.s~define('FOO', 'return "s-new"')
do i = 1 to 400
  .d~define('BAR', 'return' i)
  .a~define('BAR', 'return' i)
  .s~define('BAR', 'return' i)
end
call filler
do i = 1 to 20
  c = .object~subclass('c' i)
  c~define('FOO', 'return "stranger' i '"')
  c~define('FOO', 'return "stranger2' i '"')
  t = c~new
end
call filler
say 'u1' d~foo a~foo s~foo
say 'u1b' d~hasMethod('BAR') a~hasMethod('BAR') s~hasMethod('BAR') .d~new~bar
exit
::class d subclass directory
::method foo
  return 'd-old'
::class a subclass array
::method foo
  return 'a-old'
::class s subclass string
::method foo
  return 's-old'
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
